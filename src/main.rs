use axum::{
    body::Body,
    extract::{FromRef, State},
    response::Response,
    routing, Form, Router,
};
use serde::Deserialize;
use serde_json::value::{to_value, Value};
use std::{collections::HashMap, env};

pub fn do_nothing_filter(value: &Value, _: &HashMap<String, Value>) -> tera::Result<Value> {
    let s = tera::try_get_value!("do_nothing_filter", "value", String, value);
    Ok(to_value(s).unwrap())
}

lazy_static::lazy_static! {
    pub static ref TEMPLATES: tera::Tera = {
        let mut tera = match tera::Tera::new("templates/*") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        tera.autoescape_on(vec![".html", ".sql"]);
        tera.register_filter("do_nothing", do_nothing_filter);
        tera
    };
}

#[derive(Clone)]
struct AppState {
    client: reqwest::Client,
}

impl FromRef<AppState> for reqwest::Client {
    fn from_ref(app_state: &AppState) -> reqwest::Client {
        app_state.client.clone()
    }
}

#[derive(Deserialize)]
struct Paste {
    text: String,
    category_id: u8,
    tags: String,
    format: u8,
    expiration: char,
    exposure: u8,
    password: String,
    burn: bool,
    name: String,
}

#[tokio::main]
async fn main() {
    let state = AppState {
        client: reqwest::Client::new(),
    };

    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let addr = format!("{}:{}", host, port);

    let app = Router::new()
        .route("/", routing::get(index))
        .route("/", routing::post(paste))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    println!("Listening at {}", addr);
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to install CTRL+C signal handler");
        })
        .await
        .unwrap();

    println!("Shutting down");
}

async fn paste(State(state): State<AppState>, Form(data): Form<Paste>) -> Response {

}

async fn index() -> Response {
    Response::builder()
        .status(200)
        .header("Content-Type", "text/html")
        .body(Body::new(
            TEMPLATES
                .render("index.html", &tera::Context::new())
                .unwrap(),
        ))
        .unwrap()
}
