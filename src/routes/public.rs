use axum::{
    body::Body,
    response::{IntoResponse, Response},
    routing, Router,
};
use tera::Context;

use crate::constants::TEMPLATES;

pub fn get_router() -> Router {
    Router::new().route("/", routing::get(index))
}

async fn index() -> impl IntoResponse {
    Response::builder()
        .status(200)
        .header("Content-Type", "text/html")
        .body(Body::new(
            TEMPLATES.render("index.html", &Context::new()).unwrap(),
        ))
        .unwrap()
}
