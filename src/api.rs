pub mod compile;
pub mod download;
pub mod judge;

use anyhow::Error;
use rocket::{
    http::{ContentType, Status},
    request::Request,
    response,
    response::{Responder, Response},
};
use rocket_contrib::{json, json::JsonValue};

pub struct ApiResponse {
    status: Status,
    json: JsonValue,
}

impl<'r> Responder<'r, 'static> for ApiResponse {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        Response::build_from(self.json.respond_to(req).unwrap())
            .status(self.status)
            .header(ContentType::JSON)
            .ok()
    }
}

impl ApiResponse {
    #[allow(dead_code)]
    fn new(status: Status, json: JsonValue) -> Self {
        Self { status, json }
    }

    #[allow(dead_code)]
    fn ok(json: JsonValue) -> Self {
        Self {
            status: Status::Ok,
            json,
        }
    }

    #[allow(dead_code)]
    fn internal_server_error(e: Error) -> Self {
        Self {
            status: Status::InternalServerError,
            json: json!({ "message": format!("{:?}", &e) }),
        }
    }

    #[allow(dead_code)]
    fn bad_request(json: JsonValue) -> Self {
        Self {
            status: Status::BadRequest,
            json,
        }
    }
}
