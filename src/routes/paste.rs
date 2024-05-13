use axum::{routing, Router};

use crate::client::Client;

pub fn get_router(client: Client) -> Router {
    Router::new()
        .route("/raw/:id", routing::get(view_raw))
        .route("/json/:id", routing::get(view_json))
        .route("/dl/:id", routing::get(view_download))
        .route("/print/:id", routing::get(view_print))
        .route("/:id", routing::get(view))
        .with_state(client)
}
