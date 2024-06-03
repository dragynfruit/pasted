use axum::{
    body::Body, extract::{Path, State}, response::{IntoResponse, Response}, routing, Json, Router
};
use tera::Context;

use crate::{
    constants::URL,
    parsers::{user::User, FromHtml as _},
    state::AppState, templates::TEMPLATES,
};

pub fn get_router(state: AppState) -> Router {
    Router::new()
        .route("/:username", routing::get(user))
        .route("/json/:username", routing::get(json_user))
        .with_state(state)
}

async fn user(State(state): State<AppState>, Path(username): Path<String>) -> impl IntoResponse {
    let dom = state.client.get_html(&format!("{URL}/u/{username}"));
    let user = User::from_html(&dom);

    Response::builder()
        .status(200)
        .header("Content-Type", "text/html")
        .body(Body::new(
            TEMPLATES
                .render(
                    "user.html",
                    &Context::from_serialize(user).unwrap(),
                )
                .unwrap(),
        ))
        .unwrap()
}

async fn json_user(State(state): State<AppState>, Path(username): Path<String>) -> Json<User> {
    let dom = state.client.get_html(&format!("{URL}/u/{username}"));
    let user = User::from_html(&dom);

    Json(user)
}