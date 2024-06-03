use axum::{
    body::Body, extract::{Path, State}, response::{IntoResponse, Response}, routing, Json, Router
};
use tera::Context;

use crate::{
    constants::URL,
    parsers::{archive::ArchivePage, FromHtml},
    state::AppState,
    templates::TEMPLATES,
};

pub fn get_router(state: AppState) -> Router {
    Router::new()
        .route("/", routing::get(archive))
        .route("/:syntax", routing::get(archive))
        .route("/json", routing::get(archive_json))
        .route("/json/:syntax", routing::get(archive_json))
        .with_state(state)
}

fn get_url(syntax: Option<Path<String>>) -> String {
    if syntax.is_some() {
        let syntax = syntax.unwrap().0;
        format!("{URL}/archive/{syntax}")
    } else {
        format!("{URL}/archive")
    }
}

async fn archive(State(state): State<AppState>, syntax: Option<Path<String>>) -> impl IntoResponse {
    let dom = state.client.get_html(&get_url(syntax));
    let archive_page = ArchivePage::from_html(&dom);

    Response::builder()
        .status(200)
        .header("Content-Type", "text/html")
        .body(Body::new(
            TEMPLATES
                .render(
                    "archive.html",
                    &Context::from_serialize(archive_page).unwrap(),
                )
                .unwrap(),
        ))
        .unwrap()
}

async fn archive_json(State(state): State<AppState>, syntax: Option<Path<String>>) -> Json<ArchivePage> {
    let dom = state.client.get_html(&get_url(syntax));
    let archive_page = ArchivePage::from_html(&dom);

    Json(archive_page)
}