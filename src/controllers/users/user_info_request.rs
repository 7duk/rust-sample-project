use validator::Validate;

#[derive(serde::Serialize, Validate, serde::Deserialize)]
pub struct UsersInfo {
    #[validate(length(min = 1, message = "Name is not blank"))]
    pub name: String,
    #[validate(email(message = "Email is invalid"))]
    pub email: String,
}
