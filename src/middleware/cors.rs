use axum::http::{HeaderValue, Request, Response};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tower::{Layer, Service};

#[derive(Clone)]
pub struct DynamicCors;

impl<S> Layer<S> for DynamicCors {
    type Service = CorsMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        CorsMiddleware { inner }
    }
}

#[derive(Clone)]
pub struct CorsMiddleware<S> {
    inner: S,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for CorsMiddleware<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    ReqBody: Send + 'static,
    ResBody: 'static,
{
    type Response = Response<ResBody>;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let origin = req.headers().get("origin").cloned();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            let mut res = inner.call(req).await?;

            if let Some(origin) = origin {
                let headers = res.headers_mut();
                headers.insert("access-control-allow-origin", origin);
                headers.insert(
                    "access-control-allow-credentials",
                    HeaderValue::from_static("true"),
                );
                headers.insert("vary", HeaderValue::from_static("Origin"));
            }

            Ok(res)
        })
    }
}
