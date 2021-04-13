mod scoring;
mod sender;

use super::ApiResponse;
use crate::{
    checker::{compile_checker, run_checker},
    command::*,
    db::DbPool,
    gcp,
    models::*,
    MAX_FILE_SIZE,
};
use anyhow::{Result, bail};
use chrono::prelude::*;
use gcp::download_checker;
use rocket::State;
use rocket_contrib::{json, json::Json};
use scoring::scoring;
use sender::send_result;
use std::{
    collections::HashMap,
    fs,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
};

#[post("/judge", format = "application/json", data = "<req>")]
pub async fn judge(req: Json<JudgeRequest>, conn: State<'_, Arc<DbPool>>) -> ApiResponse {
    let conn = Arc::clone(&conn);

    if let Err(e) = download_checker(&req.0.problem.checker_path, "checker.cpp").await {
        return ApiResponse::internal_server_error(e);
    }

    // TODO download checker source and confirm testlib.h location and checker temporary location
    let checker_source_path: PathBuf = crate::JUDGE_DIR.join("checker.cpp");
    let checker_target_path: PathBuf = crate::JUDGE_DIR.join("checker");
    let testlib_path: PathBuf = PathBuf::from("/testlib.h");
    match compile_checker(&checker_source_path, &checker_target_path, &testlib_path) {
        Ok(_) => (),
        // TODO confirm error message (may not be internal server error)
        Err(e) => return ApiResponse::internal_server_error(e),
    };

    let mut submit_result = match try_testcases(&req.0, conn.clone(), &checker_target_path).await {
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
        let testcase_data = gcp::download_testcase(&req.problem.uuid, &testcase.name).await?;

        let mut file = File::create(&crate::JUDGE_DIR.join("testcase.txt"))?;
        file.write_all(&testcase_data.0)?;

        let cmd_result = exec_cmd(&req.cmd, req.time_limit).await?;
        if !cmd_result.ok {
            bail!(cmd_result.message)
        }
        dbg!(&cmd_result);
        let user_output = fs::read(&crate::JUDGE_DIR.join("userStdout.txt"))?;

        let status = judging(
            &cmd_result,
            req.time_limit,
            req.mem_limit,
            &String::from_utf8(testcase_data.0)?,
            &String::from_utf8(user_output)?,
            &String::from_utf8(testcase_data.1)?,
            checker_path,
        )?;

        let testcase_result = TestcaseResult { status, cmd_result };

        update_result(&mut submit_result, &testcase_result);

        insert_testcase_result(conn, req.submit_id, testcase.testcase_id, &testcase_result).await?;
        testcase_result_map.insert(testcase.testcase_id, testcase_result);
    }

    submit_result.testcase_result_map = testcase_result_map;

    Ok(submit_result)
}

/// Update judge result based on testcase results. Returns `true` if any fields are updated.
fn update_result(submit_result: &mut JudgeResponse, testcase_result: &TestcaseResult) -> bool {
    let mut updated = false;

    if submit_result.status.to_priority() < testcase_result.status.to_priority() {
        submit_result.status = testcase_result.status;
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
) -> Result<Status> {
    if !cmd_result.ok {
        return Ok(Status::RE);
    }
    if cmd_result.execution_time > time_limit {
        return Ok(Status::TLE);
    }
    // TODO Sandbox に output limit を渡す
    if cmd_result.stdout_size > MAX_FILE_SIZE {
        return Ok(Status::OLE);
    }
    if cmd_result.execution_memory > mem_limit {
        return Ok(Status::MLE);
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
        INSERT INTO testcase_results (submit_id, testcase_id, status, execution_time, execution_memory, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#
    )
    .bind(submit_id)
    .bind(testcase_id)
    .bind(testcase_result.status.to_string())
    .bind(testcase_result.cmd_result.execution_time)
    .bind(testcase_result.cmd_result.execution_memory)
    .bind(Local::now().naive_local())
    .bind(Local::now().naive_local())
    .execute(conn)
    .await?;

    Ok(())
}

/*
async fn get_testcases(problem_id: u64) -> Result<Vec<Testcase>> {
    let conn = db::new_pool(&CONFIG).await?;

    let testcases: Vec<Testcase> = sqlx::query_as(
        r#"
        SELECT * FROM testcases
        WHERE problem_id = ? AND deleted_at IS NULL
        "#,
    )
    .bind(problem_id)
    .fetch_all(&conn)
    .await?;

    Ok(testcases)
}

async fn get_problem(problem_id: u64) -> Result<Problem> {
    let conn = db::new_pool(&CONFIG).await?;

    let problems: Problem = sqlx::query_as(
        r#"
        SELECT * FROM problems
        id = ? AND deleted_at IS NULL
        "#,
    )
    .bind(problem_id)
    .fetch_one(&conn)
    .await?;

    Ok(problems)
}
*/
