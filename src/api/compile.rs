use std::sync::Arc;

use super::ApiResponse;
use crate::{
    command::exec_cmd,
    db::DbPool,
    models::{CompileRequest, CompileResponse, JudgeResponse},
};
use anyhow::Result;
use rocket::State;
use rocket_contrib::{json, json::Json};

#[post("/compile", format = "application/json", data = "<req>")]
pub async fn compile(req: Json<CompileRequest>, conn: State<'_, Arc<DbPool>>) -> ApiResponse {
    let conn = Arc::clone(&conn);

    let cmd_res = match exec_cmd(&req.cmd, 20_000).await {
        Ok(cmd_res) => cmd_res,
        Err(e) => return ApiResponse::internal_server_error(e),
    };

    if !cmd_res.ok {
        if let Err(e) = send_ce_result(conn, req.submit_id, &cmd_res.message).await {
            return ApiResponse::internal_server_error(e)
        }
    }

    let resp = CompileResponse(cmd_res);

    ApiResponse::ok(json!(resp))
}

pub async fn send_ce_result(conn: Arc<DbPool>, submit_id: i64, msg: &str) -> Result<()> {
    let conn = Arc::as_ref(&conn);

    sqlx::query(
        r#"
        UPDATE submits
        SET
            status = 'CE'
            , compile_error = ? 
            , point = 0
        WHERE id = ? AND deleted_at IS NULL
        "#,
    )
    .bind(msg)
    .bind(submit_id)
    .execute(conn)
    .await?;

    Ok(())
}
