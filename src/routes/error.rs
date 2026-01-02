use axum::{
    body::Body,
    http::{StatusCode, HeaderValue},
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

// Comprehensive application error type
#[derive(Debug)]
pub enum AppError {
    Client(ClientError),
    Template(tera::Error),
    Database(sled::Error),
    Io(std::io::Error),
    Server(String),
    Parser(String),
    DateParse(chrono::ParseError),
    Custom { status: u16, message: String },
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Client(err) => write!(f, "Client error: {}", err),
            AppError::Template(err) => write!(f, "Template error: {}", err),
            AppError::Database(err) => write!(f, "Database error: {}", err),
            AppError::Io(err) => write!(f, "IO error: {}", err),
            AppError::Server(msg) => write!(f, "Server error: {}", msg),
            AppError::Parser(msg) => write!(f, "Parser error: {}", msg),
            AppError::DateParse(err) => write!(f, "Date parsing error: {}", err),
            AppError::Custom { message, .. } => write!(f, "{}", message),
        }
    }
}

impl std::error::Error for AppError {}

impl From<ClientError> for AppError {
    fn from(err: ClientError) -> Self {
        AppError::Client(err)
    }
}

impl From<tera::Error> for AppError {
    fn from(err: tera::Error) -> Self {
        AppError::Template(err)
    }
}

impl From<sled::Error> for AppError {
    fn from(err: sled::Error) -> Self {
        AppError::Database(err)
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err)
    }
}

impl From<chrono::ParseError> for AppError {
    fn from(err: chrono::ParseError) -> Self {
        AppError::DateParse(err)
    }
}

impl From<AppError> for Error {
    fn from(err: AppError) -> Self {
        match err {
            AppError::Client(client_err) => Error::from(client_err),
            AppError::Template(template_err) => Error::from(template_err),
            AppError::Database(db_err) => Error {
                status: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                message: "Database operation failed".to_string(),
                details: Some(db_err.to_string()),
                stack_trace: if cfg!(debug_assertions) {
                    Some(format!("{:?}", db_err))
                } else {
                    None
                },
                source: ErrorSource::Internal,
            },
            AppError::Io(io_err) => Error {
                status: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                message: "IO operation failed".to_string(),
                details: Some(io_err.to_string()),
                stack_trace: if cfg!(debug_assertions) {
                    Some(format!("{:?}", io_err))
                } else {
                    None
                },
                source: ErrorSource::Internal,
            },
            AppError::Server(msg) => Error {
                status: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                message: msg,
                details: None,
                stack_trace: None,
                source: ErrorSource::Internal,
            },
            AppError::Parser(msg) => Error {
                status: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                message: format!("Parsing failed: {}", msg),
                details: None,
                stack_trace: None,
                source: ErrorSource::Internal,
            },
            AppError::DateParse(parse_err) => Error {
                status: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                message: "Date parsing failed".to_string(),
                details: Some(parse_err.to_string()),
                stack_trace: if cfg!(debug_assertions) {
                    Some(format!("{:?}", parse_err))
                } else {
                    None
                },
                source: ErrorSource::Internal,
            },
            AppError::Custom { status, message } => Error {
                status,
                message,
                details: None,
                stack_trace: None,
                source: ErrorSource::Internal,
            },
        }
    }
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
                    ureq::Error::StatusCode(code) => (code, "".to_string()),
                    ureq::Error::Http(transport) => {
                        (StatusCode::BAD_GATEWAY.as_u16(), transport.to_string())
                    }
                    _ => (0, "Unknown".to_string()),
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
            },
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

/// Helper function to create a fallback error response without using unwrap
pub fn create_fallback_response(message: &str) -> Response<Body> {
    let mut response = Response::new(Body::from(message.to_string()));
    *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
    response.headers_mut().insert(
        "Content-Type",
        HeaderValue::from_static("text/html")
    );
    response
}

pub fn render_error(error: Error) -> Response<Body> {
    let context_result = Context::from_serialize(&error);
    let template_result = match context_result {
        Ok(context) => TEMPLATES.render("error.html", &context),
        Err(serialize_err) => {
            eprintln!("Failed to serialize error context: {}", serialize_err);
            Ok(format!("Error {} - {}", error.status, error.message))
        }
    };

    let body = match template_result {
        Ok(rendered) => rendered,
        Err(template_err) => {
            eprintln!("Failed to render error template: {}", template_err);
            format!("Error {} - {}", error.status, error.message)
        }
    };

    Response::builder()
        .status(error.status)
        .header("Content-Type", "text/html")
        .body(Body::new(body))
        .unwrap_or_else(|err| {
            eprintln!("Failed to build error response: {}", err);
            create_fallback_response("Internal server error")
        })
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
