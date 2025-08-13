use sqlx::FromRow;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Users{
    pub id: i32,
    pub name: String,
    pub email: String,
    pub username: String,
    #[serde(skip_serializing)]
    pub password: String,
}
