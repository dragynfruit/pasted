use axum::{
    Form, Router, body::Body, extract::State, http::StatusCode, response::Response, routing,
};
use serde::Deserialize;
use tera::Context;

use crate::{constants::URL, parsers::paste, state::AppState, templates::TEMPLATES};

use super::error::{self, Error, ErrorSource, render_error};

#[derive(Deserialize)]
struct Post {
    text: String,
    category: u8,
    tags: String,
    format: u16,
    expiration: String,
    exposure: u8,
    password: String,
    title: String,
}

pub fn get_router(state: AppState) -> Router {
    Router::new()
        .route("/", routing::get(post).post(post_create))
        .with_state(state)
}

async fn post() -> Result<Response<Body>, Response<Body>> {
    TEMPLATES
        .render("post.html", &Context::new())
        .map(|html| {
            Response::builder()
                .status(200)
                .header("Content-Type", "text/html")
                .header("Cache-Control", "public, max-age=31536000, immutable")
                .body(Body::new(html))
                .unwrap_or_else(|e| {
                    eprintln!("Failed to build post response: {}", e);
                    Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::from("Internal server error"))
                        .unwrap()
                })
        })
        .map_err(|e| render_error(Error::from(e)))
}

async fn post_create(
    State(state): State<AppState>,
    Form(data): Form<Post>,
) -> Result<Response<Body>, Response<Body>> {
    let csrf = state
        .client
        .get_html(format!("{URL}/").as_str())
        .map_err(|e| error::construct_error(e))?;

    let csrf = paste::get_csrftoken(&csrf);

    let form: Vec<(String, String)> = vec![
        ("_csrf-frontend".to_string(), csrf),
        ("PostForm[text]".to_string(), data.text),
        (
            "PostForm[category_id]".to_string(),
            data.category.to_string(),
        ),
        ("PostForm[tag]".to_string(), data.tags),
        ("PostForm[format]".to_string(), data.format.to_string()),
        (
            "PostForm[expiration]".to_string(),
            data.expiration.to_string(),
        ),
        ("PostForm[status]".to_string(), data.exposure.to_string()),
        (
            "PostForm[is_password_enabled]".to_string(),
            (if data.password.is_empty() { "0" } else { "1" }).to_string(),
        ),
        ("PostForm[password]".to_string(), data.password),
        (
            "PostForm[is_burn]".to_string(),
            (if data.expiration == "B" { "1" } else { "0" }).to_string(),
        ),
        ("PostForm[name]".to_string(), data.title),
        ("PostForm[is_guest]".to_string(), "1".to_string()),
    ];

    let response = state
        .client
        .post_response(format!("{URL}/").as_str(), form)
        .map_err(|e| error::construct_error(e))?;

    let paste_id = response
        .headers()
        .get("Location")
        .ok_or_else(|| {
            render_error(Error::new(
                StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                "Missing Location header".to_string(),
                ErrorSource::Internal,
            ))
        })?
        .to_str()
        .map_err(|e| {
            render_error(Error::new(
                StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                format!("Invalid Location header encoding: {}", e),
                ErrorSource::Internal,
            ))
        })?
        .split("/")
        .last()
        .ok_or_else(|| {
            render_error(Error::new(
                StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                "Invalid Location header".to_string(),
                ErrorSource::Internal,
            ))
        })?;

    Ok(Response::builder()
        .status(response.status())
        .header("Location", format!("/{paste_id}"))
        .header("Content-Type", "text/html")
        .body(Body::empty())
        .map_err(|e| {
            render_error(Error::new(
                StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                format!("Failed to build redirect response: {}", e),
                ErrorSource::Internal,
            ))
        })?)
}
