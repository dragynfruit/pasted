use axum::Router;

use crate::client::Client;

mod images;
mod info;
mod post;
mod public;
mod users;
mod view;

pub fn get_router(client: Client) -> Router {
    Router::new()
        .nest("/", public::get_router())
        .nest("/", post::get_router(client.clone()))
        .nest("/", view::get_router(client.clone()))
        .nest("/info", info::get_router())
        .nest("/u", users::get_router(client.clone()))
        .nest("/imgs", images::get_router(client.clone()))
}
