use axum::{
    body::Body, extract::Path, response::Response, routing, Form, Router
};
use serde::Deserialize;
use serde_json::value::{to_value, Value};
use std::{collections::HashMap, env};
use serde_json::json;

const URL: &str = "https://pastebin.com";

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

#[derive(Deserialize)]
struct Paste {
    text: String,
    category: u8,
    tags: String,
    format: u8,
    expiration: char,
    exposure: u8,
    password: String,
    name: String,
}

#[tokio::main]
async fn main() {
    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let addr = format!("{}:{}", host, port);

    let app = Router::new()
        .route("/", routing::get(index).post(post))
        .route("/:id", routing::get(view));

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

async fn get_csrftoken(client: &reqwest::Client) -> String {
    let main = client.get(URL.to_owned() + "/").send().await.unwrap();
    let body = main.text().await.unwrap();
    let dom = tl::parse(body.as_str(), tl::ParserOptions::default()).unwrap();
    let parser = dom.parser();
    dom.query_selector("meta[name=csrf-token]")
        .unwrap()
        .next()
        .unwrap()
        .get(parser)
        .unwrap()
        .as_tag()
        .unwrap()
        .attributes()
        .get("content")
        .unwrap()
        .unwrap()
        .as_utf8_str()
        .into_owned()
}

async fn post(Form(data): Form<Paste>) -> Response {
    let client = reqwest::Client::builder().cookie_store(true).build().unwrap();
    let csrf = get_csrftoken(&client).await;

    let form = reqwest::multipart::Form::new()
        .text("_csrf-frontend", csrf)
        .text("PostForm[text]", data.text)
        .text("PostForm[category_id]", data.category.to_string())
        .text("PostForm[tag]", data.tags)
        .text("PostForm[format]", data.format.to_string())
        .text("PostForm[expiration]", data.expiration.to_string())
        .text("PostForm[status]", data.exposure.to_string())
        .text(
            "PostForm[is_password_enabled]",
            if data.password.is_empty() { "0" } else { "1" },
        )
        .text("PostForm[password]", data.password)
        .text(
            "PostForm[is_burn]",
            if data.expiration == 'B' { "1" } else { "0" },
        )
        .text("PostForm[name]", data.name);

    let response = client
        .post(URL.to_owned() + "/")
        .multipart(form)
        .send()
        .await
        .unwrap();
    let paste_id = response.headers().get("Location").unwrap().to_str().unwrap().split("/").last().unwrap();

    Response::builder()
        .status(response.status())
        .header("Location", format!("/{}", paste_id))
        .header("Content-Type", "text/html")
        .body(Body::new(
            TEMPLATES
                .render("index.html", &tera::Context::new())
                .unwrap(),
        ))
        .unwrap()
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

async fn view(Path(id): Path<String>) -> Response {
    let client = reqwest::Client::builder().cookie_store(true).build().unwrap();
    let response = client.get(URL.to_owned() + "/raw/" + &id).send().await.unwrap();
    let content = response.text().await.unwrap();

    Response::builder()
        .status(200)
        .header("Content-Type", "text/html")
        .body(Body::new(
            TEMPLATES
                .render("view.html", &tera::Context::from_value(json!({ "content": content })).unwrap())
                .unwrap(),
        ))
        .unwrap()
}