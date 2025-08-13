extern crate core;

mod domain;
mod controllers;
mod middlewares;

use actix_web::{web, App, HttpServer};
use std::env;
use actix_cors::Cors;
use actix_web::http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use sqlx::PgPool;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use crate::middlewares::authentication::Authentication;
use crate::middlewares::logging::{Logging};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    let env_filter= tracing_subscriber::EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(env_filter)
        .init();

    //Config database
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url).await.expect("Error connecting to database");

    // Routing
    HttpServer::new(move|| {
        let cors = Cors::default()
            .allowed_origin("http://localhost:8080")
            .allowed_methods(vec!["POST"])
            .allowed_headers(vec![AUTHORIZATION,ACCEPT,CONTENT_TYPE])
            .max_age(3600);

        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(actix_web::middleware::Logger::default())
            .wrap(cors)
            .wrap(Authentication)
            .wrap(Logging)
            .service(controllers::users::users_controller::hey)
            .service(controllers::users::users_controller::hello)
            .service(controllers::users::users_controller::show)
            .service(controllers::users::users_controller::detail)
            .service(controllers::users::users_controller::create)
            .service(controllers::authentication::authentication_controller::sign_in)
    }).bind("127.0.0.1:8080")?
        .run().await
}
