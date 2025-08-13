use serde::Serialize;
use actix_web::http::StatusCode;


#[derive(Serialize)]
pub struct SuccessResponse<T> {
    code: u16,
    message: String,
    data: T,
}

impl<T> SuccessResponse <T>{
    pub fn new(message: String, data: T) -> SuccessResponse<T> {
        SuccessResponse{code:StatusCode::OK.as_u16(),message,data}
    }
}