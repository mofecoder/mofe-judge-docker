use std::sync::Arc;

use super::ApiResponse;
use crate::{
    command::exec_compile_cmd,
    db::DbPool,
    models::{CompileRequest, CompileResponse},
    MAX_STDERR_SIZE
};
use anyhow::Result;
use rocket::State;
use rocket_contrib::{json, json::Json};

#[post("/compile", format = "application/json", data = "<req>")]
pub async fn compile(req: Json<CompileRequest>, conn: State<'_, Arc<DbPool>>) -> ApiResponse {
    let conn = Arc::clone(&conn);

    let cmd_res = match exec_compile_cmd(&req.cmd, 20).await {
        Ok(cmd_res) => cmd_res,
        Err(e) => return ApiResponse::internal_server_error(e),
    };

    dbg!(&cmd_res);

    if !cmd_res.ok {
        let user_stderr_u8 =
            std::fs::read(&crate::JUDGE_DIR.join("userStderr.txt")).unwrap_or_else(|_| Vec::new());
        let user_stderr = String::from_utf8_lossy(&user_stderr_u8[..MAX_STDERR_SIZE.min(user_stderr_u8.len())]);
        if let Err(e) = send_ce_result(conn, req.submit_id, &user_stderr).await {
            return ApiResponse::internal_server_error(e);
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
            , execution_time = NULL
            , execution_memory = NULL
        WHERE id = ? AND deleted_at IS NULL
        "#,
    )
    .bind(msg)
    .bind(submit_id)
    .execute(conn)
    .await?;

    Ok(())
}
