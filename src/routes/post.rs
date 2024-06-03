use axum::{
    body::Body,
    extract::State,
    response::{IntoResponse, Response},
    routing, Form, Router,
};
use serde::Deserialize;
use tera::Context;
use ureq_multipart::MultipartBuilder;

use crate::{
    constants::URL, state::AppState, templates::TEMPLATES, parsers::paste::get_csrftoken
};

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

async fn post() -> impl IntoResponse {
    Response::builder()
        .status(200)
        .header("Content-Type", "text/html")
        
        .header("Cache-Control", "public, max-age=31536000, immutable")
        .body(Body::new(
            TEMPLATES.render("post.html", &Context::new()).unwrap(),
        ))
        .unwrap()
}

async fn post_create(State(state): State<AppState>, Form(data): Form<Post>) -> impl IntoResponse {
    let csrf = get_csrftoken(&state.client.get_html(format!("{URL}/").as_str()));

    let form = MultipartBuilder::new()
        .add_text("_csrf-frontend", &csrf)
        .unwrap()
        .add_text("PostForm[text]", &data.text)
        .unwrap()
        .add_text("PostForm[category_id]", &data.category.to_string())
        .unwrap()
        .add_text("PostForm[tag]", &data.tags)
        .unwrap()
        .add_text("PostForm[format]", &data.format.to_string())
        .unwrap()
        .add_text("PostForm[expiration]", &data.expiration.to_string())
        .unwrap()
        .add_text("PostForm[status]", &data.exposure.to_string())
        .unwrap()
        .add_text(
            "PostForm[is_password_enabled]",
            if data.password.is_empty() { "0" } else { "1" },
        )
        .unwrap()
        .add_text("PostForm[password]", &data.password)
        .unwrap()
        .add_text(
            "PostForm[is_burn]",
            if data.expiration == "B" { "1" } else { "0" },
        )
        .unwrap()
        .add_text("PostForm[name]", &data.title)
        .unwrap()
        .add_text("PostForm[is_guest]", "1")
        .unwrap()
        .finish()
        .unwrap();

    let response = state.client.post_response(format!("{URL}/").as_str(), form);
    let paste_id = response
        .header("Location")
        .unwrap()
        .split("/")
        .last()
        .unwrap();

    Response::builder()
        .status(response.status())
        .header("Location", format!("/{paste_id}"))
        .header("Content-Type", "text/html")
        .body(Body::empty())
        .unwrap()
}
