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

pub struct CsrfProtection;

impl<S, B> Transform<S, ServiceRequest> for CsrfProtection
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = CsrfMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(CsrfMiddleware { service })
    }
}

pub struct CsrfMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for CsrfMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = Either<CsrfFuture<S, B>, Ready<Result<Self::Response, Self::Error>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        if req.method() == "POST" {
            let session_id = req.cookie("session_id").map(|c| c.value().to_string());
            let csrf_token = req.headers().get("x-csrf-token").and_then(|h| h.to_str().ok()).map(String::from);

            if let (Some(session_id), Some(csrf_token)) = (session_id, csrf_token) {
                let repo = AuthRepository::new();
                if repo.validate_csrf(&session_id, &csrf_token).is_err() {
                    let res = HttpResponse::Unauthorized()
                        .json(serde_json::json!({
                            "error": "Invalid CSRF token"
                        }));
                    return Either::right(ok(req.into_response(res)
                        .map_into_boxed_body()
                        .map_into_right_body()));
                }
            } else {
                let res = HttpResponse::Unauthorized()
                    .json(serde_json::json!({
                        "error": "Missing CSRF token or session"
                    }));
                return Either::right(ok(req.into_response(res)
                    .map_into_boxed_body()
                    .map_into_right_body()));
            }
        }

        Either::left(CsrfFuture {
          fut: self.service.call(req),
          _phantom: PhantomData,
        })
    }
} 

#[pin_project]
pub struct CsrfFuture<S, B>
where
  S: Service<ServiceRequest>,
{
  #[pin]
  fut: S::Future,
  _phantom: PhantomData<B>,
}

impl<S, B> Future for CsrfFuture<S, B>
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