use axum::Router;

use crate::state::AppState;

pub fn get_router(state: AppState) -> Router {
    Router::new().with_state(state)
}
