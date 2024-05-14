use axum::Router;

use crate::client::Client;

mod images;
mod info;
mod post;
mod public;
mod view;

pub fn get_router(client: Client) -> Router {
    Router::new()
        .nest("/", public::get_router())
        .nest("/", post::get_router(client))
        .nest("/", view::get_router(client))
        .nest("/info", info::get_router())
        .nest("/imgs", images::get_router(client))
}
