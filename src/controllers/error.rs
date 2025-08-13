use actix_web::http::StatusCode;
use actix_web::{HttpResponse, error::ResponseError};
use serde::Serialize;
use sqlx::Error as SqlxError;
use std::borrow::Cow;
use thiserror::Error;
use validator::ValidationErrors;

#[derive(Debug, Error)]
pub enum MyError {
    #[error("unauthorized")]
    Unauthorized,

    #[error("internal error")]
    InternalError,

    #[error("not found")]
    NotFound,

    #[error("validation error")]
    ValidationError(ValidationErrors),

    #[error("database error")]
    DatabaseError(SqlxError),
}

#[derive(Serialize)]
struct ErrorResponse {
    code: u16,
    message: String,
}

#[derive(Serialize)]
struct ValidationErrorResponse {
    code: u16,
    message: String,
    errors: serde_json::Value,
}

#[derive(Serialize)]
#[serde(untagged)]
enum ApiResponse {
    ValidationError(ValidationErrorResponse),
    GeneralError(ErrorResponse),
}

impl ResponseError for MyError {
    fn status_code(&self) -> StatusCode {
        match self {
            MyError::Unauthorized => StatusCode::UNAUTHORIZED,
            MyError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            MyError::NotFound => StatusCode::NOT_FOUND,
            MyError::ValidationError(_) => StatusCode::BAD_REQUEST,
            MyError::DatabaseError(_) => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let body = match self {
            MyError::ValidationError(errors) => {
                let error_json =
                    serde_json::to_value(errors).unwrap_or_else(|_| serde_json::json!({}));
                ApiResponse::ValidationError(ValidationErrorResponse {
                    code: self.status_code().as_u16(),
                    message: self.to_string(),
                    errors: error_json,
                })
            }
            MyError::DatabaseError(errors) => {
                let error_response = if let Some(db_err) = errors.as_database_error() {
                    // Error code 23505: unique constraint violation.
                    if db_err.code() == Some(Cow::from("23505")) {
                        db_err.message().to_string()
                    } else {
                        errors.to_string()
                    }
                } else {
                    String::from("unknown database error")
                };
                ApiResponse::GeneralError(ErrorResponse {
                    code: self.status_code().as_u16(),
                    message: error_response,
                })
            }
            _ => ApiResponse::GeneralError(ErrorResponse {
                code: self.status_code().as_u16(),
                message: self.to_string(),
            }),
        };

        HttpResponse::build(StatusCode::OK).json(body)
    }
}
