use crate::{
    constants::URL,
    parsers::{
        FromHtml as _,
        paste::{self, Paste},
    },
    state::AppState,
    templates::TEMPLATES,
};
use axum::{
    Form, Json, Router,
    body::Body,
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing,
};
use serde::{Deserialize, Serialize};
use tera::Context;

use super::error::{self, AppError, Error as PasteError, ErrorSource};

// Helper function to render templates safely
fn safe_render_template<T: serde::Serialize>(
    template_name: &str,
    context: &T,
) -> Result<String, AppError> {
    let ctx = Context::from_serialize(context).map_err(|e| AppError::Template(e))?;
    TEMPLATES
        .render(template_name, &ctx)
        .map_err(|e| AppError::Template(e))
}

// Helper function to create HTML responses
fn create_html_response(content: String, status: u16) -> Result<Response<Body>, AppError> {
    Response::builder()
        .status(status)
        .header("Content-Type", "text/html")
        .body(Body::from(content))
        .map_err(|e| AppError::Server(format!("Failed to build response: {}", e)))
}

// Helper function to safely parse paste from HTML
fn parse_paste_safe(dom: &scraper::Html) -> Result<Paste, Response<Body>> {
    Paste::from_html(dom).map_err(|e| {
        error::render_error(PasteError::new(
            500,
            format!("Failed to parse paste: {}", e),
            ErrorSource::Internal,
        ))
    })
}

#[derive(Serialize)]
struct Page {
    id: String,
}

#[derive(Serialize)]
struct LockScreen {
    id: String,
    burn: bool,
}

#[derive(Deserialize)]
struct Unlock {
    password: Option<String>,
}

pub fn get_router(state: AppState) -> Router {
    Router::new()
        .route("/raw/{id}", routing::get(view_raw))
        .route("/json/{id}", routing::get(view_json))
        .route("/dl/{id}", routing::get(view_download))
        .route("/print/{id}", routing::get(view_print))
        .route("/clone/{id}", routing::get(view_clone))
        .route("/embed/{id}", routing::get(view_embed))
        .route("/embed_js/{id}", routing::get(view_embed_js))
        .route("/embed_iframe/{id}", routing::get(view_embed_iframe))
        .route("/{id}", routing::get(view).post(view_locked))
        .with_state(state)
}

async fn view_raw(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let content = state.client.get_string(format!("{URL}/raw/{id}").as_str());

    match content {
        Ok(content) => match Response::builder()
            .status(200)
            .header("Content-Type", "text/plain")
            .body(Body::from(content))
        {
            Ok(response) => response,
            Err(e) => error::render_error(PasteError::new(
                500,
                format!("Failed to build response: {}", e),
                ErrorSource::Internal,
            )),
        },
        Err(err) => error::construct_error(err),
    }
}

async fn view_json(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    match state.client.get_html(format!("{URL}/{id}").as_str()) {
        Ok(dom) => {
            let paste = match parse_paste_safe(&dom) { Ok(p) => p, Err(e) => return e };
            Json(paste).into_response()
        }
        Err(err) => error::construct_error(err),
    }
}

async fn view_download(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let content = state.client.get_string(format!("{URL}/raw/{id}").as_str());

    match content {
        Ok(content) => match Response::builder()
            .status(200)
            .header("Content-Type", "text/plain")
            .header(
                "Content-Disposition",
                format!("attachment; filename=\"{id}.txt\""),
            )
            .body(Body::from(content))
        {
            Ok(response) => response,
            Err(e) => error::render_error(PasteError::new(
                500,
                format!("Failed to build download response: {}", e),
                ErrorSource::Internal,
            )),
        },
        Err(err) => error::construct_error(err),
    }
}

async fn view_print(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let dom = state.client.get_html(format!("{URL}/{id}").as_str());

    match dom {
        Ok(dom) => {
            let paste = match parse_paste_safe(&dom) { Ok(p) => p, Err(e) => return e };
            match safe_render_template("print.html", &paste) {
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

async fn view_clone(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let dom = state.client.get_html(format!("{URL}/{id}").as_str());

    match dom {
        Ok(dom) => {
            let paste = match parse_paste_safe(&dom) { Ok(p) => p, Err(e) => return e };
            match safe_render_template("post.html", &paste) {
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

async fn view_embed(Path(id): Path<String>) -> impl IntoResponse {
    let page = Page { id };
    match safe_render_template("embed.html", &page) {
        Ok(rendered) => match create_html_response(rendered, 200) {
            Ok(response) => response,
            Err(app_err) => error::render_error(PasteError::from(app_err)),
        },
        Err(app_err) => error::render_error(PasteError::from(app_err)),
    }
}

async fn view_embed_js(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let dom = state.client.get_html(format!("{URL}/{id}").as_str());

    match dom {
        Ok(dom) => {
            let paste = match parse_paste_safe(&dom) { Ok(p) => p, Err(e) => return e };
            match safe_render_template("embed_iframe.html", &paste) {
                Ok(rendered) => {
                    let js_content =
                        format!("document.write('{}');", rendered.replace('\'', "\\'"));
                    match Response::builder()
                        .status(200)
                        .header("Content-Type", "text/javascript")
                        .body(Body::from(js_content))
                    {
                        Ok(response) => response,
                        Err(e) => error::render_error(PasteError::new(
                            500,
                            format!("Failed to build JS response: {}", e),
                            ErrorSource::Internal,
                        )),
                    }
                }
                Err(app_err) => error::render_error(PasteError::from(app_err)),
            }
        }
        Err(err) => error::construct_error(err),
    }
}

async fn view_embed_iframe(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let dom = state.client.get_html(format!("{URL}/{id}").as_str());

    match dom {
        Ok(dom) => {
            let paste = match parse_paste_safe(&dom) { Ok(p) => p, Err(e) => return e };
            match safe_render_template("embed_iframe.html", &paste) {
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

async fn view_locked(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Form(data): Form<Unlock>,
) -> impl IntoResponse {
    let csrf = match state.client.get_html(format!("{URL}/{id}").as_str()) {
        Ok(dom) => paste::get_csrftoken(&dom).unwrap_or_default(),
        Err(err) => return error::construct_error(err),
    };

    let form = vec![
        ("_csrf-frontend".to_string(), csrf),
        (
            "PostPasswordVerificationForm[password]".to_string(),
            data.password.unwrap_or_default(),
        ),
        ("is_burn".to_string(), "1".to_string()),
    ];

    let dom = state.client.post_html(format!("{URL}/{id}").as_str(), form);

    match dom {
        Ok(dom) => {
            let paste = match parse_paste_safe(&dom) { Ok(p) => p, Err(e) => return e };
            match safe_render_template("view.html", &paste) {
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

async fn view(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let dom = match state.client.get_html(format!("{URL}/{id}").as_str()) {
        Ok(dom) => dom,
        Err(err) => return error::construct_error(err),
    };

    let rendered = if paste::is_locked(&dom) {
        let lock_screen = LockScreen {
            id,
            burn: paste::is_burn(&dom),
        };
        match safe_render_template("locked.html", &lock_screen) {
            Ok(content) => content,
            Err(app_err) => return error::render_error(PasteError::from(app_err)),
        }
    } else if paste::is_burn(&dom) {
        let page = Page { id };
        match safe_render_template("burn.html", &page) {
            Ok(content) => content,
            Err(app_err) => return error::render_error(PasteError::from(app_err)),
        }
    } else {
        let paste = match parse_paste_safe(&dom) { Ok(p) => p, Err(e) => return e };
        match safe_render_template("view.html", &paste) {
            Ok(content) => content,
            Err(app_err) => return error::render_error(PasteError::from(app_err)),
        }
    };

    match create_html_response(rendered, 200) {
        Ok(response) => response,
        Err(app_err) => error::render_error(PasteError::from(app_err)),
    }
}
