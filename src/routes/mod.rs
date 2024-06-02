use axum::Router;

use crate::state::AppState;

mod error;
mod imgs;
mod info;
mod post;
mod public;
mod users;
mod view;

pub fn get_router(state: AppState) -> Router {
    Router::new()
        .nest("/info", info::get_router(state.clone()))
        .nest("/u", users::get_router(state.clone()))
        .nest("/imgs", imgs::get_router(state.clone()))
        .nest("/", post::get_router(state.clone()))
        .nest("/", public::get_router())
        .nest("/", view::get_router(state.clone()))
        .fallback(error::error_404)
}
