use axum::Router;

use crate::state::AppState;

pub mod info;
mod archive;
mod error;
mod imgs;
mod post;
mod public;
mod users;
mod view;

pub fn get_router(state: AppState) -> Router {
    Router::new()
        .nest("/info", info::get_router(state.clone()))
        .nest("/archive", archive::get_router(state.clone()))
        .nest("/u", users::get_router(state.clone()))
        .nest("/imgs", imgs::get_router(state.clone()))
        .merge(post::get_router(state.clone()))
        .merge(public::get_router())
        .merge(view::get_router(state.clone()))
        .fallback(error::error_404)
}
