use super::ApiResponse;
use crate::{gcp::download_submit_source, models::DownloadRequest};
use rocket_contrib::{json, json::Json};

#[post("/download", format = "application/json", data = "<req>")]
async fn download(req: Json<DownloadRequest>) -> ApiResponse {
    if let Err(e) = download_submit_source(&req.0.code_path, &req.0.filename).await {
        return ApiResponse::internal_server_error(e);
    };

    ApiResponse::ok(json! {{}})
}
