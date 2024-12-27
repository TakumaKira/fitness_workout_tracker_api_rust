use std::{
    marker::PhantomData, pin::Pin, task::{Context, Poll}
};
use pin_project::pin_project;
use actix_utils::future::{ok, Either, Ready};
use actix_web::{
    body::{EitherBody, MessageBody},
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse,
};
use crate::repositories::auth_repository::AuthRepository;
use futures::{ready, Future};

pub struct SessionProtection;

impl<S, B> Transform<S, ServiceRequest> for SessionProtection
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = SessionMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(SessionMiddleware { service })
    }
}

pub struct SessionMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for SessionMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = Either<SessionFuture<S, B>, Ready<Result<Self::Response, Self::Error>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        if let Some(session) = req.cookie("session_id") {
            let repo = AuthRepository::new();
            if repo.validate_session(&session.value().to_string()).is_err() {
                let res = HttpResponse::Unauthorized()
                    .json(serde_json::json!({
                        "error": "Invalid session"
                    }));
                return Either::right(ok(req.into_response(res)
                    .map_into_boxed_body()
                    .map_into_right_body()));
            }
            Either::left(SessionFuture {
                fut: self.service.call(req),
                _phantom: PhantomData,
            })
        } else {
            let res = HttpResponse::Unauthorized()
                .json(serde_json::json!({
                    "error": "Authentication required"
                }));
            Either::right(ok(req.into_response(res)
                .map_into_boxed_body()
                .map_into_right_body()))
        }
    }
}

#[pin_project]
pub struct SessionFuture<S, B>
where
    S: Service<ServiceRequest>,
{
    #[pin]
    fut: S::Future,
    _phantom: PhantomData<B>,
}

impl<S, B> Future for SessionFuture<S, B>
where
    B: MessageBody,
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
{
    type Output = Result<ServiceResponse<EitherBody<B>>, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let res = match ready!(self.project().fut.poll(cx)) {
            Ok(res) => res,
            Err(err) => return Poll::Ready(Err(err.into())),
        };

        Poll::Ready(Ok(res.map_into_left_body()))
    }
} 