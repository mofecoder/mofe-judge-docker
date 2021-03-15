mod scoring;
mod sender;

use super::ApiResponse;
use crate::{MAX_FILE_SIZE, command::*, db::DbPool, gcp, models::*};
use anyhow::Result;
use chrono::prelude::*;
use rocket::State;
use rocket_contrib::{json, json::Json};
use scoring::scoring;
use sender::send_result;
use std::{collections::HashMap, fs, fs::File, io::Write, sync::Arc};

#[post("/judge", format = "application/json", data = "<req>")]
pub async fn judge(req: Json<JudgeRequest>, conn: State<'_, Arc<DbPool>>) -> ApiResponse {    
    let conn = Arc::clone(&conn);

    let mut submit_result = match try_testcases(&req.0, conn.clone()).await {
        Ok(submit_result) => submit_result,
        Err(e) => return ApiResponse::internal_server_error(e),
    };

    submit_result.score = match scoring(conn.clone(), &req, &submit_result).await {
        Ok(score) => score,
        Err(e) => return ApiResponse::internal_server_error(e),
    };

    if let Err(e) = send_result(conn.clone(), &submit_result).await {
        return ApiResponse::internal_server_error(e);
    }

    ApiResponse::ok(json!(submit_result))
}

async fn try_testcases(req: &JudgeRequest, conn: Arc<DbPool>) -> Result<JudgeResponse> {
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

        let mut file = File::create("testcase.txt")?;
        file.write_all(&testcase_data.0)?;

        let cmd_result = exec_cmd(&req.cmd, req.time_limit).await?;
        let user_output = fs::read("userStdout.txt")?;

        let status = judging(
            &cmd_result,
            req.time_limit,
            req.mem_limit,
            &String::from_utf8(user_output)?,
            &String::from_utf8(testcase_data.0)?,
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
    user_output: &str,
    testcase_output: &str,
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

    // TODO: checker に user_output と testcase_output を渡す
    if user_output.trim() == testcase_output.trim() {
        Ok(Status::AC)
    } else {
        Ok(Status::WA)
    }
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
        INSERT INTO testcase_results (submit_id, testcase_id, status, execution_time, execution_memory, created_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#
    )
    .bind(submit_id)
    .bind(testcase_id)
    .bind(testcase_result.status.to_string())
    .bind(testcase_result.cmd_result.execution_time)
    .bind(testcase_result.cmd_result.execution_memory)
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
