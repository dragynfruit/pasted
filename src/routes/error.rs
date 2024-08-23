use axum::{
    body::Body,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use tera::Context;

use crate::client::ClientError;
use crate::templates::TEMPLATES;

#[derive(Serialize)]
pub struct Error {
    status: u16,
    message: Option<String>,
}

pub fn render_error(error: Error) -> Response<Body> {
    Response::builder()
        .status(error.status)
        .header("Content-Type", "text/html")
        .body(Body::new(
            TEMPLATES
                .render("error.html", &Context::from_serialize(&error).unwrap())
                .unwrap(),
        ))
        .unwrap()
}

pub async fn error_404() -> impl IntoResponse {
    return render_error(Error {
        status: StatusCode::NOT_FOUND.as_u16(),
        message: None,
    });
}

pub fn construct_error(error: ClientError) -> Response<Body> {
    match error {
        ClientError::UreqError(error) => {
            let response = error.into_response().unwrap();
            render_error(Error {
                status: response.status(),
                message: None,
            })
            .into_response()
        }
        ClientError::IoError(error) => render_error(Error {
            status: 500,
            message: Some(error.to_string()),
        })
        .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_error_404() {
        let response = error_404().await;
        assert_eq!(response.into_response().status(), StatusCode::NOT_FOUND);
    }
}
