use axum::{
    body::Body,
    http::StatusCode,
    response::Response,
    routing, Router,
};

use super::error::{Error, ErrorSource, render_error};

pub fn get_router() -> Router {
    Router::new()
        .route("/favicon.png", routing::get(favicon_png))
        .route("/favicon.ico", routing::get(favicon_ico))
        .route("/manifest.json", routing::get(manifest))
        .route("/robots.txt", routing::get(robots))
}

async fn favicon_png() -> Result<Response<Body>, Response<Body>> {
    Response::builder()
        .status(200)
        .header("Content-Type", "image/png")
        .header("Cache-Control", "public, max-age=31536000, immutable")
        .body(Body::from(include_bytes!("assets/favicon.png").to_vec()))
        .map_err(|_e| render_error(Error::new(
            StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            "Failed to serve favicon.png".to_string(),
            ErrorSource::Internal
        )))
}

async fn favicon_ico() -> Result<Response<Body>, Response<Body>> {
    Response::builder()
        .status(200)
        .header("Content-Type", "image/x-icon")
        .header("Cache-Control", "public, max-age=31536000, immutable")
        .body(Body::from(include_bytes!("assets/favicon.ico").to_vec()))
        .map_err(|_e| render_error(Error::new(
            StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            "Failed to serve favicon.ico".to_string(),
            ErrorSource::Internal
        )))
}

async fn manifest() -> Result<Response<Body>, Response<Body>> {
    Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .header("Cache-Control", "public, max-age=31536000, immutable")
        .body(Body::from(include_bytes!("assets/manifest.json").to_vec()))
        .map_err(|_e| render_error(Error::new(
            StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            "Failed to serve manifest.json".to_string(),
            ErrorSource::Internal
        )))
}

async fn robots() -> Result<Response<Body>, Response<Body>> {
    Response::builder()
        .status(200)
        .header("Content-Type", "text/plain")
        .header("Cache-Control", "public, max-age=31536000, immutable")
        .body(Body::from(include_bytes!("assets/robots.txt").to_vec()))
        .map_err(|_e| render_error(Error::new(
            StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            "Failed to serve robots.txt".to_string(),
            ErrorSource::Internal
        )))
}

#[cfg(test)]
mod tests {
    use axum::response::IntoResponse as _;

    use super::*;

    #[tokio::test]
    async fn test_favicon_png() {
        let response = favicon_png().await.unwrap();
        assert_eq!(response.into_response().status(), 200);
    }

    #[tokio::test]
    async fn test_favicon_ico() {
        let response = favicon_ico().await.unwrap();
        assert_eq!(response.into_response().status(), 200);
    }

    #[tokio::test]
    async fn test_manifest() {
        let response = manifest().await.unwrap();
        assert_eq!(response.into_response().status(), 200);
    }

    #[tokio::test]
    async fn test_robots() {
        let response = robots().await.unwrap();
        assert_eq!(response.into_response().status(), 200);
    }
}