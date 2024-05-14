use axum::{
    body::Body,
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing, Router,
};

use crate::{client::Client, constants::URL};

pub fn get_router(client: Client) -> Router {
    Router::new()
        .route("/favicon.ico", routing::get(favicon))
        .route("/imgs/guest.png", routing::get(guest))
        .route("/imgs/:id0/:id1/:id2/:id3.jpg", routing::get(icon))
        .with_state(client)
}

async fn favicon() -> impl IntoResponse {
    Response::builder()
        .status(200)
        .header("Content-Type", "image/x-icon")
        .header("Cache-Control", "public, max-age=31536000, immutable")
        .body(Body::from(include_bytes!("assets/favicon.ico").to_vec()))
        .unwrap()
}

async fn guest() -> impl IntoResponse {
    Response::builder()
        .status(200)
        .header("Content-Type", "image/png")
        .header("Cache-Control", "public, max-age=31536000, immutable")
        .body(Body::from(include_bytes!("assets/guest.png").to_vec()))
        .unwrap()
}

async fn icon(
    State(client): State<Client>,
    Path((id0, id1, id2, id3)): Path<(String, String, String, String)>,
) -> impl IntoResponse {
    let id3 = id3.split_once(".").unwrap().0;
    let icon = client.get_bytes(format!("{URL}/cache/img/{id0}/{id1}/{id2}/{id3}.jpg").as_str());

    Response::builder()
        .status(200)
        .header("Content-Type", "image/jpeg")
        .header("Cache-Control", "public, max-age=31536000, immutable")
        .body(Body::from(icon))
        .unwrap()
}
