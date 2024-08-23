use axum::{
    body::Body,
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing, Json, Router,
};
use tera::Context;

use crate::{
    constants::URL,
    parsers::{archive::ArchivePage, FromHtml},
    state::AppState,
    templates::TEMPLATES,
};

use super::error;

pub fn get_router(state: AppState) -> Router {
    Router::new()
        .route("/", routing::get(archive))
        .route("/:format", routing::get(archive))
        .route("/json", routing::get(archive_json))
        .route("/json/:format", routing::get(archive_json))
        .with_state(state)
}

fn get_url(format: Option<Path<String>>) -> String {
    if format.is_some() {
        let format = format.unwrap().0;
        format!("{URL}/archive/{format}")
    } else {
        format!("{URL}/archive")
    }
}

async fn archive(State(state): State<AppState>, format: Option<Path<String>>) -> impl IntoResponse {
    let dom = state.client.get_html(&get_url(format));

    match dom {
        Ok(dom) => Response::builder()
            .status(200)
            .header("Content-Type", "text/html")
            .body(Body::new(
                TEMPLATES
                    .render(
                        "archive.html",
                        &Context::from_serialize(ArchivePage::from_html(&dom)).unwrap(),
                    )
                    .unwrap(),
            ))
            .unwrap(),
        Err(err) => error::construct_error(err),
    }
}

async fn archive_json(
    State(state): State<AppState>,
    format: Option<Path<String>>,
) -> Json<ArchivePage> {
    let dom = state.client.get_html(&get_url(format)).unwrap(); //fix
    let archive_page = ArchivePage::from_html(&dom);

    Json(archive_page)
}
