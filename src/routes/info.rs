use axum::{Json, Router, body::Body, extract::State, http::StatusCode, response::Response, routing};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use tera::Context;

use super::error::{Error, render_error};
use crate::{state::AppState, templates::TEMPLATES};

pub static DEPLOY_DATE: OnceLock<String> = OnceLock::new();

#[derive(Deserialize, Serialize, Clone)]
struct InstanceInfo {
    version: &'static str,
    name: &'static str,
    is_release: bool,
    db_size: u64,
    commit: &'static str,
    action_name: &'static str,
    build_date: &'static str,
    deploy_date: &'static str,
    static_templates: bool,
}

fn get_info(state: AppState) -> InstanceInfo {
    // let commit = include_str!("../../.git/FETCH_HEAD")
    //     .lines()
    //     .next()
    //     .ok_or_else(|| Error::new(
    //         StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
    //         "Failed to read commit info".to_string(),
    //         ErrorSource::Internal
    //     ))
    //     .unwrap()
    //     .split('\t')
    //     .next()
    //     .unwrap();
    // use none
    let commit = "Currently Broken";

    let build_date = env!("BUILD_DATE");
    let deploy_date = DEPLOY_DATE.get().map(|s| s.as_str()).unwrap_or("Unknown");

    InstanceInfo {
        version: env!("CARGO_PKG_VERSION"),
        name: env!("CARGO_PKG_NAME"),
        is_release: !cfg!(debug_assertions),
        db_size: state.db.size_on_disk().unwrap_or(0),
        action_name: env!("ACTION_NAME"),
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

async fn info(State(state): State<AppState>) -> Result<Response<Body>, Response<Body>> {
    let info = get_info(state);
    let context = match Context::from_serialize(&info) {
        Ok(ctx) => ctx,
        Err(e) => return Err(render_error(Error::from(e))),
    };

    TEMPLATES
        .render("info.html", &context)
        .map(|html| {
            Response::builder()
                .status(200)
                .header("Content-Type", "text/html")
                .body(Body::new(html))
                .unwrap_or_else(|e| {
                    eprintln!("Failed to build info response: {}", e);
                    // Final fallback - construct response manually
                    let mut response = Response::new(Body::from("Internal server error"));
                    *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                    response
                })
        })
        .map_err(|e| render_error(Error::from(e)))
}

async fn info_json(State(state): State<AppState>) -> Result<Json<InstanceInfo>, Response<Body>> {
    let info = get_info(state);
    Ok(Json(info))
}
