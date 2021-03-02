use super::ApiResponse;
use crate::model::*;
use crate::command::*;
use rocket_contrib::json;
use rocket_contrib::json::Json;

#[post("/compilation", format = "application/json", data = "<req>")]
async fn compilation(req: Json<RequestJson>) -> ApiResponse {
    let res = match exec_cmd(&req.0).await {
        Ok(cmd_res) => cmd_res,
        Err(e) => return ApiResponse::internal_server_error(e),
    };

    ApiResponse::ok(json! {{}})
}

