use super::ApiResponse;
use crate::command::exec_cmd;
use rocket_contrib::{json, json::Json};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct RequestJson {
    pub submit_id: i64,
    pub cmd: String, // コンパイルコマンド or 実行コマンド
}

#[post("/compile", format = "application/json", data = "<req>")]
async fn compile(req: Json<RequestJson>) -> ApiResponse {
    let cmd_res = match exec_cmd(&req.cmd, 20_000).await {
        Ok(cmd_res) => cmd_res,
        Err(e) => return ApiResponse::internal_server_error(e),
    };

    ApiResponse::ok(json! {cmd_res})
}
