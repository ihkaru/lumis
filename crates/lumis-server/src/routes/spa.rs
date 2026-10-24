use axum::body::Body;
use axum::http::{header, Method, Request, Response, StatusCode};
use std::convert::Infallible;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower_http::services::ServeDir;

#[derive(Clone)]
pub struct SpaFallbackService {
    index_file: PathBuf,
}

impl SpaFallbackService {
    pub fn new(index_file: PathBuf) -> Self {
        Self { index_file }
    }
}

impl<ReqBody> tower::Service<Request<ReqBody>> for SpaFallbackService
where
    ReqBody: Send + 'static,
{
    type Response = Response<Body>;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let index_path = self.index_file.clone();
        let method = req.method().clone();
        let path = req.uri().path().to_string();

        Box::pin(async move {
            // 1. Only GET/HEAD can receive index.html
            if method != Method::GET && method != Method::HEAD {
                let res = Response::builder()
                    .status(StatusCode::METHOD_NOT_ALLOWED)
                    .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
                    .body(Body::from("Method Not Allowed"))
                    .unwrap_or_default();
                return Ok(res);
            }

            // 2. API & system endpoints must 404 with JSON, never return index.html
            if path.starts_with("/api/")
                || path.starts_with("/v1/")
                || path.starts_with("/scim/")
                || path == "/health"
                || path == "/health/db"
                || path == "/metrics"
            {
                let res = Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(r#"{"error":"Endpoint not found"}"#))
                    .unwrap_or_default();
                return Ok(res);
            }

            // 3. Missing assets or paths with file extension must 404 to prevent MIME errors
            let has_extension = path
                .rsplit('/')
                .next()
                .map(|s| s.contains('.') && !s.starts_with('.'))
                .unwrap_or(false);

            if path.starts_with("/assets/") || has_extension {
                let res = Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
                    .body(Body::from("Resource not found"))
                    .unwrap_or_default();
                return Ok(res);
            }

            // 4. SPA Deep-Linking: serve index.html for client routes (/chat/:id, /login, /embed/*, etc.)
            if let Ok(content) = tokio::fs::read(&index_path).await {
                let mut builder = Response::builder()
                    .status(StatusCode::OK)
                    .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                    .header(header::CACHE_CONTROL, "no-cache");

                if path.starts_with("/embed") {
                    let ancestors = std::env::var("EMBED_ALLOWED_ANCESTORS")
                        .unwrap_or_else(|_| "'self' *".to_string());
                    builder = builder.header(
                        "Content-Security-Policy",
                        format!("frame-ancestors {};", ancestors),
                    );
                }

                let res = builder
                    .body(Body::from(content))
                    .unwrap_or_default();
                Ok(res)
            } else {
                let res = Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
                    .body(Body::from("Index file not found"))
                    .unwrap_or_default();
                Ok(res)
            }
        })
    }
}

pub fn create_spa_service(static_dir: impl AsRef<std::path::Path>) -> ServeDir<SpaFallbackService> {
    let static_path = static_dir.as_ref().to_path_buf();
    let index_file = static_path.join("index.html");
    ServeDir::new(static_path).fallback(SpaFallbackService::new(index_file))
}
