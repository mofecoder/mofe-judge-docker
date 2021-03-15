use crate::{db::DbPool, models::JudgeResponse};
use anyhow::Result;
use std::sync::Arc;

pub async fn send_result(db_conn: Arc<DbPool>, submit_result: &JudgeResponse) -> Result<()> {
    let db_conn = Arc::as_ref(&db_conn);

    sqlx::query(
        r#"
        UPDATE submits
        SET
            status = ?
            , execution_time = ?
            , execution_memory = ?
            , point = ?
        WHERE id = ? AND deleted_at IS NULL
        "#,
    )
    .bind(&submit_result.status.to_string())
    .bind(submit_result.execution_time)
    .bind(submit_result.execution_memory)
    .bind(submit_result.score)
    .bind(submit_result.submit_id)
    .execute(db_conn)
    .await?;

    Ok(())
}
