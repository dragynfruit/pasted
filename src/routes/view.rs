use crate::{
    client::Client,
    constants::URL,
    templates::TEMPLATES,
    paste::{self, Paste},
};
use axum::{
    body::Body,
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing, Json, Router,
};
use tera::Context;

pub fn get_router(client: Client) -> Router {
    Router::new()
        .route("/raw/:id", routing::get(view_raw))
        .route("/json/:id", routing::get(view_json))
        .route("/dl/:id", routing::get(view_download))
        .route("/print/:id", routing::get(view_print))
        .route("/:id", routing::get(view))
        .with_state(client)
}

async fn view_raw(State(client): State<Client>, Path(id): Path<String>) -> impl IntoResponse {
    let content = client.get_string(format!("{URL}/raw/{id}").as_str());

    Response::builder()
        .status(200)
        .header("Content-Type", "text/plain")
        .body(Body::from(content))
        .unwrap()
}

async fn view_json(State(client): State<Client>, Path(id): Path<String>) -> Json<Paste> {
    let dom = client.get_html(format!("{URL}/{id}").as_str());
    let paste = paste::parse_paste(&dom);

    Json(paste)
}

async fn view_download(State(client): State<Client>, Path(id): Path<String>) -> impl IntoResponse {
    let content = client.get_string(format!("{URL}/raw/{id}").as_str());

    Response::builder()
        .status(200)
        .header("Content-Type", "text/plain")
        .header(
            "Content-Disposition",
            format!("attachment; filename=\"{id}.txt\""),
        )
        .body(Body::from(content))
        .unwrap()
}

async fn view_print(State(client): State<Client>, Path(id): Path<String>) -> impl IntoResponse {
    let dom = client.get_html(format!("{URL}/{id}").as_str());
    let paste = paste::parse_paste(&dom);

    Response::builder()
        .status(200)
        .header("Content-Type", "text/html")
        .body(Body::from(
            TEMPLATES
                .render("print.html", &Context::from_serialize(paste).unwrap())
                .unwrap(),
        ))
        .unwrap()
}

async fn view(State(client): State<Client>, Path(id): Path<String>) -> impl IntoResponse {
    let dom = client.get_html(format!("{URL}/{id}").as_str());
    let paste = paste::parse_paste(&dom);

    Response::builder()
        .status(200)
        .header("Content-Type", "text/html")
        .body(Body::from(
            TEMPLATES
                .render("view.html", &Context::from_serialize(paste).unwrap())
                .unwrap(),
        ))
        .unwrap()
        .into_response()
}
