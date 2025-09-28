use axum::{
    body::Body, extract::{Path, State}, http::StatusCode, response::Response, routing, Router
};

use crate::{state::AppState, constants::URL};

use super::error::{render_error, Error, ErrorSource};

pub fn get_router(state: AppState) -> Router {
    Router::new()
        .route("/guest.png", routing::get(guest))
        .route("/{id0}/{id1}/{id2}/{id3}", routing::get(icon))
        .with_state(state)
}

async fn guest() -> Result<Response<Body>, Response<Body>> {
    Response::builder()
        .status(200)
        .header("Content-Type", "image/png")
        .header("Cache-Control", "public, max-age=31536000, immutable")
        .body(Body::from(include_bytes!("assets/guest.png").to_vec()))
        .map_err(|_e| render_error(Error::new(
            StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            "Failed to serve guest image".to_string(),
            ErrorSource::Internal
        )))
}

async fn icon(
    State(state): State<AppState>,
    Path((id0, id1, id2, id3)): Path<(String, String, String, String)>,
) -> Result<Response<Body>, Response<Body>> {
    let id3 = id3.split_once(".")
        .ok_or_else(|| render_error(Error::new(
            StatusCode::BAD_REQUEST.as_u16(),
            "Invalid image path".to_string(),
            ErrorSource::Internal
        )))?
        .0;

    let path = format!("{id0}/{id1}/{id2}/{id3}");
    let tree = state.db.open_tree("icons").unwrap();

    let icon = if tree.contains_key(&path).unwrap() {
        tree.get(&path).unwrap().unwrap().to_vec()
    } else {
        let icon = state.client.get_bytes(format!("{URL}/cache/img/{path}.jpg").as_str()).unwrap();
        let save_icon = icon.clone();
        tokio::spawn(async move {
            tree.insert(&path, save_icon).ok();
        });
        icon
    };

    Response::builder()
        .status(200)
        .header("Content-Type", "image/jpeg")
        .header("Cache-Control", "public, max-age=31536000, immutable")
        .body(Body::from(icon))
        .map_err(|_e| render_error(Error::new(
            StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            "Failed to serve icon image".to_string(),
            ErrorSource::Internal
        )))
}

#[cfg(test)]
mod tests {
    use axum::response::IntoResponse as _;

    use super::*;

    #[tokio::test]
    async fn test_guest() {
        let response = guest().await.unwrap();
        assert_eq!(response.into_response().status(), 200);
    }

    #[tokio::test]
    async fn test_icon() {
        let state = AppState::default();
        let response = icon(
            State(state.clone()),
            Path((
                "22".to_string(),
                "20".to_string(),
                "25".to_string(),
                "10674139.jpg".to_string(),
            )),
        )
        .await
        .unwrap();
        assert_eq!(response.into_response().status(), 200);
    }
}
