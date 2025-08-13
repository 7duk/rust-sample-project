use crate::controllers::error::MyError;
use crate::controllers::response::SuccessResponse;
use crate::controllers::users::user_info_request::UsersInfo;
use crate::domain::users_entity::Users;
use actix_web::{HttpResponse, Responder, get, post, web};
use log::info;
use serde_json::json;
use sqlx::PgPool;
use std::collections::HashMap;
use validator::Validate;

#[get("/hey/{name}")]
async fn hey(path: web::Path<String>) -> impl Responder {
    let name = path.into_inner();
    HttpResponse::Ok().body(format!("Hey {name}!"))
}

#[get("/hello")]
async fn hello(query: web::Query<HashMap<String, String>>) ->  Result<impl Responder,MyError> {
    let query_values = query.into_inner();
    let name = query_values.get("name");
    match name {
        Some(name) => Ok(format!("Hello {name}!")),
        None => Err(MyError::NotFound.into()),
    }
}

#[get("/users")]
async fn show(pool: web::Data<PgPool>) -> Result<impl Responder, MyError> {
    let users = sqlx::query_as!(Users, "SELECT * FROM users")
        .fetch_all(pool.get_ref())
        .await
        .map_err(|_| MyError::InternalError)?;

    let res = SuccessResponse::new(String::default(), users);
    Ok(HttpResponse::Ok().json(res))
}

#[get("/users/{id}")]
async fn detail(pool: web::Data<PgPool>, path: web::Path<i32>) -> Result<HttpResponse, MyError> {
    let id = path.into_inner();
    info!("User ID: {id}");
    let user = sqlx::query_as::<_, Users>("select * from users where id = $1")
        .bind(id)
        .fetch_optional(pool.get_ref())
        .await
        .map_err(|_err| MyError::InternalError)?;

    match user {
        Some(user) => {
            let res = SuccessResponse::new(String::default(), user);
            Ok(HttpResponse::Ok().json(res))
        }
        None => Err(MyError::NotFound),
    }
}

#[post("/users")]
async fn create(
    pool: web::Data<PgPool>,
    user: web::Json<UsersInfo>,
) -> Result<impl Responder, MyError> {
    user.validate()
        .map_err(|error| MyError::ValidationError(error))?;

    let user_created = sqlx::query!(
        "insert into users(name, email) values ( $1, $2) returning id",
        &user.name,
        &user.email
    )
    .fetch_one(pool.get_ref())
    .await
    .map_err(|err| MyError::DatabaseError(err))?;

    let res = SuccessResponse::new("created".to_string(), json!({ "id": user_created.id }));
    Ok(HttpResponse::Ok().json(res))
}
