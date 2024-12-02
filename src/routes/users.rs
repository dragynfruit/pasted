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

use super::error::{self, render_error, Error};

pub fn get_router(state: AppState) -> Router {
    Router::new()
        .route("/:username", routing::get(user))
        .route("/json/:username", routing::get(json_user))
        .with_state(state)
}

async fn user(State(state): State<AppState>, Path(username): Path<String>) -> impl IntoResponse {
    let dom = state.client.get_html(&format!("{URL}/u/{username}"));
    match dom {
        Ok(dom) => {
            let user = User::from_html(&dom);
            match TEMPLATES.render("user.html", &Context::from_serialize(&user).unwrap() ) {
                Ok(html) => Response::builder()
                    .status(200)
                    .header("Content-Type", "text/html")
                    .body(Body::new(html))
                    .unwrap(),
                Err(err) => render_error(Error::from(err)),
            }
        }
        Err(err) => error::construct_error(err),
    }
}

async fn json_user(
    State(state): State<AppState>, 
    Path(username): Path<String>
) -> Result<Json<User>, Response<Body>> {
    let dom = state.client.get_html(&format!("{URL}/u/{username}"))
        .map_err(|e| error::construct_error(e))?;
    
    let user = User::from_html(&dom);
    Ok(Json(user))
}
