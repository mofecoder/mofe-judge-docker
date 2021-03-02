use super::ApiResponse;
use crate::model::*;
use crate::command::*;
use anyhow::Result;
use rocket_contrib::json;
use rocket_contrib::json::Json;
use std::{fs::File, io::Write};
use crate::gcp;

#[post("/judge", format = "application/json", data = "<req>")]
async fn judge(req: Json<RequestJson>) -> ApiResponse {
    let res = match exec_cmd(&req.0).await {
        Ok(cmd_res) => cmd_res,
        Err(e) => return ApiResponse::internal_server_error(e),
    };

    ApiResponse::ok(json! {{}})
}

async fn try_testcase(req: &RequestJson) -> Result<()> {
    let testcase = gcp::download_testcase(&req.problem.uuid, &req.testcase.name).await?;

    let mut file = File::create("testcase.txt")?;
    file.write_all(&testcase.0)?;
    
    let cmd_res = exec_cmd(&req).await?;

    Ok(())
}

async fn get_testcases(problem_id: u64) -> Result<Vec<Testcase>> {

}

async fn get_problem(id: u64) -> Result<Problem> {
    
}