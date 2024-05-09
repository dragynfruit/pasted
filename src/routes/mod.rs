use axum::Router;

mod paste;
mod info;
mod images;

pub fn get_router() -> Router {
    Router::new()
        .nest("/", paste::get_router())
        .nest("/info", info::get_router())
        .nest("/imgs", images::get_router())
}