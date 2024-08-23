use axum::{
    body::Body,
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing, Json, Router,
};
use tera::Context;

use crate::{
    constants::URL,
    parsers::{user::User, FromHtml as _},
    state::AppState,
    templates::TEMPLATES,
};

use super::error;

pub fn get_router(state: AppState) -> Router {
    Router::new()
        .route("/:username", routing::get(user))
        .route("/json/:username", routing::get(json_user))
        .with_state(state)
}

async fn user(State(state): State<AppState>, Path(username): Path<String>) -> impl IntoResponse {
    let dom = state.client.get_html(&format!("{URL}/u/{username}"));

    match dom {
        Ok(dom) => Response::builder()
            .status(200)
            .header("Content-Type", "text/html")
            .body(Body::new(
                TEMPLATES
                    .render(
                        "user.html",
                        &Context::from_serialize(User::from_html(&dom)).unwrap(),
                    )
                    .unwrap(),
            ))
            .unwrap(),
        Err(err) => error::construct_error(err),
    }
}

async fn json_user(State(state): State<AppState>, Path(username): Path<String>) -> Json<User> {
    let dom = state.client.get_html(&format!("{URL}/u/{username}")).unwrap(); // fix
    let user = User::from_html(&dom);

    Json(user)
}
