use axum::{
    body::Body,
    extract::State,
    response::{IntoResponse, Response},
    routing, Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use tera::Context;

use crate::{state::AppState, templates::TEMPLATES};

pub static DEPLOY_DATE: OnceLock<String> = OnceLock::new();

#[derive(Deserialize, Serialize, Clone)]
struct InstanceInfo {
    version: &'static str,
    name: &'static str,
    is_release: bool,
    db_size: u64,
    commit: &'static str,
    used_actions: bool,
    build_date: &'static str,
    deploy_date: &'static str,
    static_templates: bool,
}

fn get_info(state: AppState) -> InstanceInfo {
    let commit = include_str!("../../.git/FETCH_HEAD")
        .lines()
        .next()
        .unwrap()
        .split('\t')
        .next()
        .unwrap();

    let build_date = env!("BUILD_DATE");
    let deploy_date = DEPLOY_DATE.get().unwrap();

    InstanceInfo {
        version: env!("CARGO_PKG_VERSION"),
        name: env!("CARGO_PKG_NAME"),
        is_release: !cfg!(debug_assertions),
        db_size: state.db.size_on_disk().unwrap(),
        used_actions: std::env::var("USED_ACTIONS").unwrap_or_default() == "true",
        commit,
        build_date,
        deploy_date,
        static_templates: cfg!(feature = "include_templates"),
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
