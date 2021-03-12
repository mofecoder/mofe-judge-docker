use super::ApiResponse;
use crate::{command::exec_cmd, models::CompileRequest};
use rocket_contrib::{json, json::Json};

#[post("/compile", format = "application/json", data = "<req>")]
async fn compile(req: Json<CompileRequest>) -> ApiResponse {
    let cmd_res = match exec_cmd(&req.cmd, 20_000).await {
        Ok(cmd_res) => cmd_res,
        Err(e) => return ApiResponse::internal_server_error(e),
    };

    ApiResponse::ok(json! {cmd_res})
}
