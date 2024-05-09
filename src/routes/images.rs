use axum::{routing, Router};

pub fn get_router() -> Router {
    Router::new()
        .route("/imgs/guest.png", routing::get(guest))
        .route("/imgs/:id0/:id1/:id2/:id3.jpg", routing::get(icon))
}