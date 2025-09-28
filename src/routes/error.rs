use axum::{
    body::Body,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use tera::Context;

use crate::client::ClientError;
use crate::templates::TEMPLATES;

#[derive(Serialize, Debug)]
pub enum ErrorSource {
    Upstream,
    Internal,
}

#[derive(Serialize, Debug)]
pub struct Error {
    status: u16,
    message: String,
    details: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stack_trace: Option<String>,
    source: ErrorSource,
}

impl Error {
    pub fn new(status: u16, message: String, source: ErrorSource) -> Self {
        Self {
            status,
            message,
            details: None,
            stack_trace: None,
            source,
        }
    }
}

impl From<ClientError> for Error {
    fn from(err: ClientError) -> Self {
        match err {
            ClientError::UreqError(error) => {
                let (status, message) = match error {
                    ureq::Error::StatusCode(code) => {
                        (code, "".to_string())
                    }
                    ureq::Error::Http(transport) => {
                        (StatusCode::BAD_GATEWAY.as_u16(), transport.to_string())
                    }
                    _ => {
                        (0, "Unknown".to_string())
                    }
                };
                Error::new(status, message, ErrorSource::Upstream)
            }
            ClientError::IoError(error) => Error {
                status: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                message: "Internal Server Error".to_string(),
                details: Some(error.to_string()),
                stack_trace: if cfg!(debug_assertions) {
                    Some(format!("{:?}", error))
                } else {
                    None
                },
                source: ErrorSource::Internal,
            }
        }
    }
}

impl From<tera::Error> for Error {
    fn from(err: tera::Error) -> Self {
        Error {
            status: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            message: "Template rendering error".to_string(),
            details: Some(err.to_string()),
            stack_trace: if cfg!(debug_assertions) {
                Some(format!("{:?}", err))
            } else {
                None
            },
            source: ErrorSource::Internal,
        }
    }
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
    render_error(Error::new(
        StatusCode::NOT_FOUND.as_u16(),
        "Page not found".to_string(),
        ErrorSource::Internal,
    ))
}

pub fn construct_error(error: ClientError) -> Response<Body> {
    render_error(Error::from(error))
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
