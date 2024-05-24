use crate::{
    client::Client,
    constants::URL,
    paste::{self, Paste},
    templates::TEMPLATES,
};
use axum::{
    body::Body,
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing, Json, Router,
};
use serde::Serialize;
use tera::Context;

#[derive(Serialize)]
struct Page {
    id: String,
}

pub fn get_router(client: Client) -> Router {
    Router::new()
        .route("/raw/:id", routing::get(view_raw))
        .route("/json/:id", routing::get(view_json))
        .route("/dl/:id", routing::get(view_download))
        .route("/print/:id", routing::get(view_print))
        .route("/clone/:id", routing::get(view_clone))
        .route("/embed/:id", routing::get(view_embed))
        .route("/embed_js/:id", routing::get(view_embed_js))
        .route("/embed_iframe/:id", routing::get(view_embed_iframe))
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

async fn view_clone(State(client): State<Client>, Path(id): Path<String>) -> impl IntoResponse {
    let dom = client.get_html(format!("{URL}/{id}").as_str());
    let paste = paste::parse_paste(&dom);

    Response::builder()
        .status(200)
        .header("Content-Type", "text/html")
        .body(Body::new(
            TEMPLATES
                .render("post.html", &Context::from_serialize(paste).unwrap())
                .unwrap(),
        ))
        .unwrap()
}

async fn view_embed(Path(id): Path<String>) -> impl IntoResponse {
    Response::builder()
        .status(200)
        .header("Content-Type", "text/html")
        .body(Body::from(
            TEMPLATES
                .render("embed.html", &Context::from_serialize(Page { id }).unwrap())
                .unwrap(),
        ))
        .unwrap()
}

async fn view_embed_js(State(client): State<Client>, Path(id): Path<String>) -> impl IntoResponse {
    let dom = client.get_html(format!("{URL}/{id}").as_str());
    let paste = paste::parse_paste(&dom);

    Response::builder()
        .status(200)
        .header("Content-Type", "text/javascript")
        .body(Body::from(format!(
            "document.write('{}');",
            TEMPLATES
                .render("embed_iframe.html", &Context::from_serialize(paste).unwrap())
                .unwrap()
        )))
        .unwrap()
}

async fn view_embed_iframe(
    State(client): State<Client>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let dom = client.get_html(format!("{URL}/{id}").as_str());
    let paste = paste::parse_paste(&dom);

    Response::builder()
        .status(200)
        .header("Content-Type", "text/html")
        .body(Body::from(
            TEMPLATES
                .render(
                    "embed_iframe.html",
                    &Context::from_serialize(paste).unwrap(),
                )
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
