use crate::controllers::error::MyError;
use actix_web::Error;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready};
use chrono::{Duration, Utc};
use futures_util::future::{LocalBoxFuture, Ready, ready};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::env;

pub struct Authentication;

pub struct AuthenticationMiddleware<S> {
    service: S,
}

impl<S, B> Transform<S, ServiceRequest> for Authentication
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthenticationMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthenticationMiddleware { service }))
    }
}

impl<S, B> Service<ServiceRequest> for AuthenticationMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let white_list = vec!["/sign-in", "/hey", "/hello"];
        if white_list.contains(&req.path()) {
            let fut = self.service.call(req);
            return Box::pin(async move { fut.await });
        }
        let token = jwt_token(&req);
        match token {
            Ok(token) => {
                let valid_token = valid_jwt_token(&token);
                if !valid_token {
                    return Box::pin(async move { Err(MyError::Unauthorized.into()) });
                }
                let fut = self.service.call(req);
                Box::pin(async move {
                    let res = fut.await?;

                    Ok(res)
                })
            }
            Err(_) => Box::pin(async move { Err(MyError::Unauthorized.into()) }),
        }
    }
}

fn jwt_token(req: &ServiceRequest) -> Result<String, MyError> {
    let auth_header = req.headers().get("Authorization");
    match auth_header {
        Some(auth_header) => {
            let bearer = auth_header.to_str().map_err(|_| MyError::Unauthorized)?;
            if !bearer.starts_with("Bearer ") {
                return Err(MyError::Unauthorized);
            }
            let token = bearer[7..].to_string();
            Ok(token)
        }
        None => Err(MyError::Unauthorized),
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}
fn extract_token(token: &str) -> Result<Claims, MyError> {
    let secret_key = env::var("TOKEN_SECRET").map_err(|_| MyError::InternalError)?;
    let decoded = decode(
        &token,
        &DecodingKey::from_secret(secret_key.as_ref()),
        &Validation::default(),
    );
    match decoded {
        Ok(decoded) => Ok(decoded.claims),
        _ => Err(MyError::Unauthorized),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Token {
    pub token: String,
    pub expires_at: usize,
}

pub fn create_token(sub: u32) -> Result<Token, MyError> {
    let secret_key = env::var("TOKEN_SECRET").map_err(|_| MyError::InternalError)?;

    let exp_str = env::var("TOKEN_EXP").map_err(|_| MyError::InternalError)?;

    let exp_duration_seconds: i64 = exp_str.parse().map_err(|_| MyError::InternalError)?;

    let expires_at = (Utc::now() + Duration::seconds(exp_duration_seconds)).timestamp() as usize;
    let my_claims = Claims {
        sub: sub.to_string(),
        exp: expires_at,
    };

    let token = encode(
        &Header::default(),
        &my_claims,
        &EncodingKey::from_secret(secret_key.as_ref()),
    )
    .map_err(|_| MyError::InternalError)?;

    Ok(Token { token, expires_at })
}

fn valid_jwt_token(token: &str) -> bool {
    match extract_token(&token) {
        Ok(claims) => {
            info!(
                "exp: {}, now: {}",
                claims.exp,
                Utc::now().timestamp() as usize
            );
            if claims.exp < Utc::now().timestamp() as usize {
                return false;
            }
            true
        }
        Err(e) => {
            error!("JWT validation error: {}", e.to_string());
            false
        }
    }
}
