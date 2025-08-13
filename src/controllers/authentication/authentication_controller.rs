use crate::controllers::authentication::user_login_request::LoginRequest;
use crate::controllers::error::MyError;
use crate::controllers::response::SuccessResponse;
use crate::domain::users_entity::Users;
use crate::middlewares::authentication::create_token;
use actix_web::HttpResponse;
use actix_web::{Responder, post, web};
use bcrypt::verify;
use sqlx::Error as SqlxError;
use sqlx::PgPool;
use validator::Validate;

#[post("/sign-in")]
async fn sign_in(
    user_sign_up: web::Json<LoginRequest>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder, MyError> {
    user_sign_up
        .validate()
        .map_err(|error| MyError::ValidationError(error))?;

    let user = sqlx::query_as!(
        Users,
        "select * from users where username = $1",
        &user_sign_up.username
    )
    .fetch_one(pool.get_ref())
    .await
    .map_err(|error| match error {
        SqlxError::RowNotFound => MyError::NotFound,
        _ => MyError::DatabaseError(error),
    })?;

    if !verify(&user_sign_up.password, &user.password).map_err(|_| MyError::Unauthorized)? {
        return Err(MyError::Unauthorized);
    }

    let token = create_token(user.id as u32).map_err(|_| MyError::InternalError)?;

    let res = SuccessResponse::new(String::default(), token);
    Ok(HttpResponse::Ok().json(res))
}
