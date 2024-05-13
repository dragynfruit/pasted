use axum::Router;

use crate::client::Client;

mod public;
mod paste;
mod info;
mod images;

pub fn get_router(client: Client) -> Router {
    Router::new()
        .nest("/", public::get_router())
        .nest("/", paste::get_router(client))
        .nest("/info", info::get_router())
        .nest("/imgs", images::get_router(client))
}