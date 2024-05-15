use axum::Router;

use crate::client::Client;

pub fn get_router(client: Client) -> Router {
    Router::new().with_state(client)
}
