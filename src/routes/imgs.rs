use axum::{
    body::Body,
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing, Router,
};

use crate::{state::AppState, constants::URL};

pub fn get_router(state: AppState) -> Router {
    Router::new()
        .route("/guest.png", routing::get(guest))
        .route("/:id0/:id1/:id2/:id3.jpg", routing::get(icon))
        .with_state(state)
}

async fn guest() -> impl IntoResponse {
    Response::builder()
        .status(200)
        .header("Content-Type", "image/png")
        .header("Cache-Control", "public, max-age=31536000, immutable")
        .body(Body::from(include_bytes!("assets/guest.png").to_vec()))
        .unwrap()
}

async fn icon(
    State(state): State<AppState>,
    Path((id0, id1, id2, id3)): Path<(String, String, String, String)>,
) -> impl IntoResponse {
    let id3 = id3.split_once(".").unwrap().0;
    let path = format!("{id0}/{id1}/{id2}/{id3}");
    let tree = state.db.open_tree("icons").unwrap();

    let icon = if tree.contains_key(&path).unwrap() {
        tree.get(&path).unwrap().unwrap().to_vec()
    } else {
        let icon = state.client.get_bytes(format!("{URL}/cache/img/{path}.jpg").as_str()).unwrap(); //fix
        tree.insert(&path, icon.clone()).unwrap();
        icon
    };

    Response::builder()
        .status(200)
        .header("Content-Type", "image/jpeg")
        .header("Cache-Control", "public, max-age=31536000, immutable")
        .body(Body::from(icon))
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_guest() {
        let response = guest().await;
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
        .await;
        assert_eq!(response.into_response().status(), 200);
    }
}
