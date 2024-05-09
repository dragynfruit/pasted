use axum::{routing, Router};

pub fn get_router() -> Router {
    Router::new()
        .route("/raw/:id", routing::get(view_raw))
        .route("/json/:id", routing::get(view_json))
        .route("/dl/:id", routing::get(view_download))
        .route("/print/:id", routing::get(view_print))
        .route("/:id", routing::get(view))
}
