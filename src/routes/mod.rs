use axum::Router;

use crate::client::Client;

mod imgs;
mod info;
mod post;
mod users;
mod view;

pub fn get_router(client: Client) -> Router {
    Router::new()
        .nest("/info", info::get_router())
        .nest("/u", users::get_router(client.clone()))
        .nest("/imgs", imgs::get_router(client.clone()))
        .nest("/", post::get_router(client.clone()))
        .nest("/", view::get_router(client.clone()))
}
