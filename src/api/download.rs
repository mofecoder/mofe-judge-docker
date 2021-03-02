use super::ApiResponse;
use crate::gcp::download_submit_source;
use rocket_contrib::{json, json::Json};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct RequestJson {
    pub submit_id: i64,
    pub code_path: String, // gcp 上のパス
    pub filename: String,  // Main.ext
}

#[post("/download", format = "application/json", data = "<req>")]
async fn download(req: Json<RequestJson>) -> ApiResponse {
    if let Err(e) = download_submit_source(&req.0.code_path, &req.0.filename).await {
        return ApiResponse::internal_server_error(e);
    };

    ApiResponse::ok(json! {{}})
}
