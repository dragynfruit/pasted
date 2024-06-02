use axum::{
    body::Body, extract::State, response::{IntoResponse, Response}, routing, Json, Router
};
use serde::{Deserialize, Serialize};
use tera::Context;

use crate::{state::AppState, templates::TEMPLATES};

#[derive(Deserialize, Serialize, Clone)]
struct InstanceInfo {
    version: String,
    name: String,
    is_release: bool,
    db_size: u64,
}

fn get_info(state: AppState) -> InstanceInfo {
    InstanceInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        name: env!("CARGO_PKG_NAME").to_string(),
        is_release: !cfg!(debug_assertions),
        db_size: state.db.size_on_disk().unwrap(),
    }
}

pub fn get_router(state: AppState) -> Router {
    Router::new()
        .route("/", routing::get(info))
        .route("/json", routing::get(info_json))
        .with_state(state)
}

async fn info(State(state): State<AppState>) -> impl IntoResponse {
    Response::builder()
        .status(200)
        .header("Content-Type", "text/html")
        .body(Body::new(
            TEMPLATES
                .render(
                    "info.html",
                    &Context::from_serialize(&get_info(state)).unwrap(),
                )
                .unwrap(),
        ))
        .unwrap()
}

async fn info_json(State(state): State<AppState>) -> Json<InstanceInfo> {
    Json(get_info(state))
}
