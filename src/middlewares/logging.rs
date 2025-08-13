use std::future::{Ready, ready};

use actix_web::{
    Error,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
};
use futures_util::future::LocalBoxFuture;
use log::info;

pub struct Logging;

pub struct LoggingMiddleware<S> {
    service: S,
}

impl<S, B> Transform<S, ServiceRequest> for Logging
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = LoggingMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(LoggingMiddleware { service }))
    }
}

impl<S, B> Service<ServiceRequest> for LoggingMiddleware<S>
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
        info!("------------------------ Middleware Logging ------------------------");
        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;

            Ok(res)
        })
    }
}
