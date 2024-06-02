use axum::{
    body::Body,
    response::{IntoResponse, Response},
    routing, Router,
};

pub fn get_router() -> Router {
    Router::new()
        .route("/favicon.png", routing::get(favicon_png))
        .route("/favicon.ico", routing::get(favicon_ico))
        .route("/manifest.json", routing::get(manifest))
        .route("/robots.txt", routing::get(robots))
}

async fn favicon_png() -> impl IntoResponse {
    Response::builder()
        .status(200)
        .header("Content-Type", "image/png")
        .header("Cache-Control", "public, max-age=31536000, immutable")
        .body(Body::from(include_bytes!("assets/favicon.png").to_vec()))
        .unwrap()
}

async fn favicon_ico() -> impl IntoResponse {
    Response::builder()
        .status(200)
        .header("Content-Type", "image/x-icon")
        .header("Cache-Control", "public, max-age=31536000, immutable")
        .body(Body::from(include_bytes!("assets/favicon.ico").to_vec()))
        .unwrap()
}

async fn manifest() -> impl IntoResponse {
    Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .header("Cache-Control", "public, max-age=31536000, immutable")
        .body(Body::from(include_bytes!("assets/manifest.json").to_vec()))
        .unwrap()
}

async fn robots() -> impl IntoResponse {
    Response::builder()
        .status(200)
        .header("Content-Type", "text/plain")
        .header("Cache-Control", "public, max-age=31536000, immutable")
        .body(Body::from(include_bytes!("assets/robots.txt").to_vec()))
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_favicon_png() {
        let response = favicon_png().await;
        assert_eq!(response.into_response().status(), 200);
    }

    #[tokio::test]
    async fn test_favicon_ico() {
        let response = favicon_ico().await;
        assert_eq!(response.into_response().status(), 200);
    }

    #[tokio::test]
    async fn test_manifest() {
        let response = manifest().await;
        assert_eq!(response.into_response().status(), 200);
    }

    #[tokio::test]
    async fn test_robots() {
        let response = robots().await;
        assert_eq!(response.into_response().status(), 200);
    }
}