use axum::{
    body::Body,
    response::{IntoResponse, Response},
    routing, Json, Router,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::time;
use tera::Context;

use crate::templates::TEMPLATES;

#[derive(Deserialize, Serialize, Clone)]
struct InstanceInfo {
    version: String,
    name: String,
    start_time: String,
    is_release: bool,
}

static INSTANCE_INFO: Lazy<InstanceInfo> = Lazy::new(|| InstanceInfo {
    version: env!("CARGO_PKG_VERSION").to_string(),
    name: env!("CARGO_PKG_NAME").to_string(),
    start_time: time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string(),
    is_release: cfg!(debug_assertions),
});

pub fn get_router() -> Router {
    Router::new()
        .route("/", routing::get(info))
        .route("/json", routing::get(info_json))
}

async fn info() -> impl IntoResponse {
    Response::builder()
        .status(200)
        .header("Content-Type", "text/html")
        .body(Body::new(
            TEMPLATES
                .render(
                    "info.html",
                    &Context::from_serialize(&*INSTANCE_INFO).unwrap(),
                )
                .unwrap(),
        ))
        .unwrap()
}

async fn info_json() -> Json<InstanceInfo> {
    Json(INSTANCE_INFO.clone())
}
