use super::ApiResponse;
use crate::gcp::GcpClient;
use crate::models::DownloadRequest;
use rocket::serde::json::{json, Json};
use rocket::State;
use std::sync::Arc;

#[post("/download", format = "application/json", data = "<req>")]
pub async fn download(req: Json<DownloadRequest>, gcp: &State<Arc<GcpClient>>) -> ApiResponse {
    eprintln!("downloading submission source...");
    let start = std::time::Instant::now();
    if let Err(e) = gcp
        .download_submit_source(&req.0.code_path, &req.0.filename)
        .await
    {
        return ApiResponse::internal_server_error(e);
    };
    eprintln!("done. took {:?}", start.elapsed());

    ApiResponse::ok(json! {{}})
}
