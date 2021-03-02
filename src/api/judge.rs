use super::ApiResponse;
use crate::{command::*, db, gcp, model::*, CONFIG, MAX_FILE_SIZE, MAX_MEMORY_USAGE};
use anyhow::Result;
use chrono::prelude::*;
use rocket_contrib::{json, json::Json};
use serde::Deserialize;
use std::{fs, fs::File, io::Write};

#[derive(Deserialize)]
pub struct RequestJson {
    pub submit_id: i64,
    pub cmd: String,     // コンパイルコマンド or 実行コマンド
    pub time_limit: i32, // 実行制限時間
    pub mem_limit: i32,  // メモリ制限

    pub testcases: Vec<Testcase>, // pub testcase: Testcase,
    pub problem: Problem,         // pub problem: Problem,
}

#[post("/judge", format = "application/json", data = "<req>")]
pub async fn judge(req: Json<RequestJson>) -> ApiResponse {
    let testcase_results = match try_testcases(&req.0).await {
        Ok(testcase_results) => testcase_results,
        Err(e) => return ApiResponse::internal_server_error(e),
    };

    // todo: scoring, send_result

    ApiResponse::ok(json!(testcase_results))
}

async fn try_testcases(req: &RequestJson) -> Result<Vec<TestcaseResult>> {
    let mut testcase_results = Vec::new();

    for testcase in &req.testcases {
        let testcase_data = gcp::download_testcase(&req.problem.uuid, &testcase.name).await?;

        let mut file = File::create("testcase.txt")?;
        file.write_all(&testcase_data.0)?;

        let cmd_result = exec_cmd(&req.cmd, req.time_limit).await?;
        let user_output = fs::read("userStdout.txt")?;

        let status = judging(
            &cmd_result,
            req.time_limit,
            &String::from_utf8(user_output)?,
            &String::from_utf8(testcase_data.0)?,
        )?;

        let testcase_result = TestcaseResult { status, cmd_result };

        insert_testcase_result(req.submit_id, testcase.testcase_id, &testcase_result).await?;
        testcase_results.push(testcase_result);
    }

    Ok(testcase_results)
}

#[allow(clippy::clippy::unnecessary_wraps, unused_variables)]
fn judging(
    cmd_result: &CmdResult,
    time_limit: i32,
    user_output: &str,
    testcase_output: &str,
) -> Result<Status> {
    if !cmd_result.ok {
        return Ok(Status::RE);
    }
    if cmd_result.time > time_limit {
        return Ok(Status::TLE);
    }
    // todo: checker に user_output と testcase_output を渡す
    if cmd_result.stdout_size > MAX_FILE_SIZE {
        return Ok(Status::OLE);
    }
    if cmd_result.mem_usage > MAX_MEMORY_USAGE {
        return Ok(Status::MLE);
    }

    Ok(Status::AC)
}

async fn insert_testcase_result(
    submit_id: i64,
    testcase_id: i64,
    testcase_result: &TestcaseResult,
) -> Result<()> {
    let conn = db::new_pool(&CONFIG).await?;

    sqlx::query(
        r#"
        INSERT INTO testcase_results (submit_id, testcase_id, status, execution_time, execution_memory, created_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#
    )
    .bind(submit_id)
    .bind(testcase_id)
    .bind(testcase_result.status.value())
    .bind(testcase_result.cmd_result.time)
    .bind(testcase_result.cmd_result.mem_usage)
    .bind(Local::now().naive_local())
    .execute(&conn)
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
