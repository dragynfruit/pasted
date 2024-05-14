use crate::{client::Client, constants::URL};
use axum::{
    body::Body,
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing, Router,
};

pub fn get_router(client: Client) -> Router {
    Router::new()
        .route("/raws/:id", routing::get(view_raw))
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
