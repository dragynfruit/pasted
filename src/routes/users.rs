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

use super::error::{self, render_error, Error, AppError};

// Helper function to render templates safely
fn safe_render_template<T: serde::Serialize>(template_name: &str, context: &T) -> Result<String, AppError> {
    let ctx = Context::from_serialize(context).map_err(|e| AppError::Template(e))?;
    TEMPLATES.render(template_name, &ctx).map_err(|e| AppError::Template(e))
}

// Helper function to create HTML responses
fn create_html_response(content: String, status: u16) -> Result<Response<Body>, AppError> {
    Response::builder()
        .status(status)
        .header("Content-Type", "text/html")
        .body(Body::from(content))
        .map_err(|e| AppError::Server(format!("Failed to build response: {}", e)))
}

pub fn get_router(state: AppState) -> Router {
    Router::new()
        .route("/{username}", routing::get(user))
        .route("/json/{username}", routing::get(json_user))
        .with_state(state)
}

async fn user(State(state): State<AppState>, Path(username): Path<String>) -> impl IntoResponse {
    let dom = state.client.get_html(&format!("{URL}/u/{username}"));
    match dom {
        Ok(dom) => {
            let user = User::from_html(&dom);
            match safe_render_template("user.html", &user) {
                Ok(rendered) => match create_html_response(rendered, 200) {
                    Ok(response) => response,
                    Err(app_err) => render_error(Error::from(app_err)),
                },
                Err(app_err) => render_error(Error::from(app_err)),
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
