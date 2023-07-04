use crate::{db::DbPool, models::JudgeResponse};
use anyhow::Result;
use std::sync::Arc;

pub async fn send_result(conn: Arc<DbPool>, submit_result: &JudgeResponse) -> Result<()> {
    let conn = Arc::as_ref(&conn);

    sqlx::query(
        r#"
        UPDATE submissions
        SET
            status = ?
            , execution_time = ?
            , execution_memory = ?
            , point = ?
            , compile_error = ''
        WHERE id = ? AND deleted_at IS NULL
        "#,
    )
    .bind(&submit_result.status.to_string())
    .bind(submit_result.execution_time)
    .bind(submit_result.execution_memory)
    .bind(submit_result.score)
    .bind(submit_result.submit_id)
    .execute(conn)
    .await?;

    Ok(())
}
