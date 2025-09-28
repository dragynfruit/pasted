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

use super::error::{self, AppError, Error as PasteError};

// Helper function to render templates safely
fn safe_render_template<T: serde::Serialize>(template_name: &str, context: &T) -> Result<String, AppError> {
    let ctx = Context::from_serialize(context).map_err(|e| AppError::Template(e))?;
    TEMPLATES.render(template_name, &ctx).map_err(|e| AppError::Template(e))
}

// Helper function to create HTML responses
fn create_html_response(content: String, status: u16) -> Result<Response<Body>, AppError> {
    Response::builder()
        .status(status)
        .header("Content-Type", "text/html")
        .body(Body::from(content))
        .map_err(|e| AppError::Server(format!("Failed to build response: {}", e)))
}

pub fn get_router(state: AppState) -> Router {
    Router::new()
        .route("/", routing::get(archive))
        .route("/{format}", routing::get(archive))
        .route("/json", routing::get(archive_json))
        .route("/json/{format}", routing::get(archive_json))
        .with_state(state)
}

fn get_url(format: Option<Path<String>>) -> String {
    if let Some(format) = format {
        let format = format.0;
        format!("{URL}/archive/{format}")
    } else {
        format!("{URL}/archive")
    }
}

async fn archive(State(state): State<AppState>, format: Option<Path<String>>) -> impl IntoResponse {
    let dom = state.client.get_html(&get_url(format));

    match dom {
        Ok(dom) => {
            let archive_page = ArchivePage::from_html(&dom);
            match safe_render_template("archive.html", &archive_page) {
                Ok(rendered) => match create_html_response(rendered, 200) {
                    Ok(response) => response,
                    Err(app_err) => error::render_error(PasteError::from(app_err)),
                },
                Err(app_err) => error::render_error(PasteError::from(app_err)),
            }
        }
        Err(err) => error::construct_error(err),
    }
}

async fn archive_json(
    State(state): State<AppState>,
    format: Option<Path<String>>,
) -> impl IntoResponse {
    match state.client.get_html(&get_url(format)) {
        Ok(dom) => {
            let archive_page = ArchivePage::from_html(&dom);
            Json(archive_page).into_response()
        }
        Err(err) => error::construct_error(err),
    }
}
