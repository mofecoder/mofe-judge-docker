mod scoring;
mod sender;

use super::ApiResponse;
use crate::gcp::GcpClient;
use crate::{
    checker::{compile_checker, run_checker},
    command::*,
    db::DbPool,
    models::*,
    MAX_FILE_SIZE,
};
use anyhow::Result;
use chrono::prelude::*;
use rocket::serde::json::{json, Json};
use rocket::State;
use scoring::scoring;
use sender::send_result;
use std::{
    collections::HashMap,
    fs,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
    time,
};

#[post("/judge", format = "application/json", data = "<req>")]
pub async fn judge(
    req: Json<JudgeRequest>,
    conn: &State<Arc<DbPool>>,
    gcp: &State<Arc<GcpClient>>,
) -> ApiResponse {
    let conn = Arc::clone(&conn);

    eprintln!("download and compiling checker...");
    let start = time::Instant::now();
    if let Err(e) = gcp
        .download_checker(&req.0.problem.checker_path, "checker.cpp")
        .await
    {
        return ApiResponse::internal_server_error(e);
    }
    eprintln!("downloaded. took {:?}", start.elapsed());

    // TODO download checker source and confirm testlib.h location and checker temporary location
    let checker_source_path: PathBuf = crate::JUDGE_DIR.join("checker.cpp");
    let checker_target_path: PathBuf = crate::JUDGE_DIR.join("checker");
    let testlib_path: PathBuf = PathBuf::from("/opt/testlib/testlib.h");
    match compile_checker(&checker_source_path, &checker_target_path, &testlib_path) {
        Ok(_) => (),
        // TODO confirm error message (may not be internal server error)
        Err(e) => return ApiResponse::internal_server_error(e),
    };
    eprintln!("done. took total {:?}", start.elapsed());

    let mut submit_result =
        match try_testcases(&req.0, conn.clone(), (*gcp).clone(), &checker_target_path).await {
            Ok(submit_result) => submit_result,
            Err(e) => return ApiResponse::internal_server_error(e),
        };

    dbg!(&submit_result);

    submit_result.score = match scoring(conn.clone(), &req, &submit_result).await {
        Ok(score) => score,
        Err(e) => return ApiResponse::internal_server_error(e),
    };

    if let Err(e) = send_result(conn.clone(), &submit_result).await {
        return ApiResponse::internal_server_error(e);
    }

    ApiResponse::ok(json!(submit_result))
}

async fn try_testcases(
    req: &JudgeRequest,
    conn: Arc<DbPool>,
    gcp: Arc<GcpClient>,
    checker_path: &Path,
) -> Result<JudgeResponse> {
    let mut submit_result = JudgeResponse {
        submit_id: req.submit_id,
        status: Status::AC,
        execution_time: 0,
        execution_memory: 0,
        score: 0,
        testcase_result_map: HashMap::new(),
    };
    let mut testcase_result_map = HashMap::new();

    for testcase in &req.testcases {
        let conn = Arc::clone(&conn);
        let testcase_data = gcp
            .download_testcase(&req.problem.uuid, &testcase.name)
            .await?;

        let mut file = File::create(&crate::JUDGE_DIR.join("testcase.txt"))?;
        file.write_all(&testcase_data.0)?;

        let cmd_result = exec_execute_cmd(&req.cmd, req.time_limit as f64 / 1000.0).await?;
        dbg!(&cmd_result);

        let user_output = fs::read(&crate::JUDGE_DIR.join("userStdout.txt"))?;
        let user_error = fs::read(&crate::JUDGE_DIR.join("userStderr.txt"))?;
        let user_output = String::from_utf8(user_output)?;
        let user_error = String::from_utf8(user_error)?;
        if !cmd_result.ok {
            println!(
                "RE[exit {}](Submission#{}/testcase#{})\nstdout: {}\nstderr: {}\n",
                cmd_result.exit_code,
                req.submit_id,
                testcase.testcase_id,
                user_output.replace("\n", "\\n"),
                user_error.replace("\n", "\\n")
            );
        }
        let testcase_result = {
            let result = judging(
                &cmd_result,
                req.time_limit,
                req.mem_limit,
                &String::from_utf8(testcase_data.0)?,
                &user_output,
                &String::from_utf8(testcase_data.1)?,
                checker_path,
            )?;

            TestcaseResult { result, cmd_result }
        };

        dbg!(&testcase_result);

        update_result(&mut submit_result, &testcase_result);
        if submit_result.status != Status::AC {
            update_submit_status(
                conn.clone(),
                req.submit_id,
                &submit_result.status.to_string(),
            )
            .await?;
        }

        insert_testcase_result(conn, req.submit_id, testcase.testcase_id, &testcase_result).await?;
        testcase_result_map.insert(testcase.testcase_id, testcase_result);
    }

    submit_result.testcase_result_map = testcase_result_map;

    Ok(submit_result)
}

/// Update judge result based on testcase results. Returns `true` if any fields are updated.
fn update_result(submit_result: &mut JudgeResponse, testcase_result: &TestcaseResult) -> bool {
    let mut updated = false;

    if testcase_result.result.status != Status::AC
        && submit_result.status.to_priority() < testcase_result.result.status.to_priority()
    {
        submit_result.status = testcase_result.result.status;
        updated = true;
    }

    if submit_result.execution_memory < testcase_result.cmd_result.execution_memory {
        submit_result.execution_memory = testcase_result.cmd_result.execution_memory;
        updated = true;
    }

    if submit_result.execution_time < testcase_result.cmd_result.execution_time {
        submit_result.execution_time = testcase_result.cmd_result.execution_time;
        updated = true;
    }

    updated
}

#[allow(clippy::clippy::unnecessary_wraps, unused_variables)]
fn judging(
    cmd_result: &CmdResult,
    time_limit: i32,
    mem_limit: i32,
    testcase_input: &str,
    user_output: &str,
    testcase_output: &str,
    checker_path: &Path,
) -> Result<JudgeResult> {
    if cmd_result.execution_time > time_limit {
        return Ok(JudgeResult::from_status(Status::TLE));
    }

    // TODO Sandbox に output limit を渡す
    if cmd_result.stdout_size > MAX_FILE_SIZE {
        return Ok(JudgeResult::from_status(Status::OLE));
    }

    if cmd_result.execution_memory > mem_limit {
        return Ok(JudgeResult::from_status(Status::MLE));
    }

    if !cmd_result.ok {
        return Ok(JudgeResult::from_status(Status::RE));
    }

    let result = run_checker(checker_path, testcase_input, user_output, testcase_output)?;

    Ok(result)
}

async fn insert_testcase_result(
    conn: Arc<DbPool>,
    submit_id: i64,
    testcase_id: i64,
    testcase_result: &TestcaseResult,
) -> Result<()> {
    let conn = Arc::as_ref(&conn);

    sqlx::query(
        r#"
        INSERT INTO testcase_results (submission_id, testcase_id, status, score, execution_time, execution_memory, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#
    )
    .bind(submit_id)
    .bind(testcase_id)
    .bind(testcase_result.result.status.to_string())
    .bind(testcase_result.result.score)
    .bind(testcase_result.cmd_result.execution_time)
    .bind(testcase_result.cmd_result.execution_memory)
    .bind(Local::now().naive_local())
    .bind(Local::now().naive_local())
    .execute(conn)
    .await?;

    Ok(())
}

async fn update_submit_status(conn: Arc<DbPool>, id: i64, status: &str) -> Result<u64> {
    let conn = Arc::as_ref(&conn);

    let result = sqlx::query!(
        r#"
        UPDATE submissions
        SET
            status = ?
        WHERE
            id = ?
        "#,
        status,
        id,
    )
    .execute(conn)
    .await?;
    Ok(result.rows_affected())
}
