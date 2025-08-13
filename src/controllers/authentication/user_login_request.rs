use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Deserialize, Validate, Serialize)]
pub struct LoginRequest {
    #[validate(length(min = 1, max = 20, message = "username is not blank"))]
    pub username: String,
    #[validate(length(min = 6, max = 20, message = "password is invalid"))]
    pub password: String,
}