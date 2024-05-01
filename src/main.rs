use axum::{body::Body, extract::Path, response::Response, routing, Form, Router};
use serde::Deserialize;
use serde_json::json;
use serde_json::value::{to_value, Value};
use std::{collections::HashMap, env};

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
struct Post {
    text: String,
    category: u8,
    tags: String,
    format: u8,
    expiration: char,
    exposure: u8,
    password: String,
    name: String,
}

#[derive(Deserialize)]
struct User {
    username: String,
    icon: String,
}

#[derive(Deserialize)]
struct Comment {
    author: User,
    text: String,
    date: String,
    likes: u32,
    dislikes: u32,
    format: String,
    link: String,
}

#[derive(Deserialize)]
struct Paste {
    author: User,
    content: String,
    unlisted: bool,
    title: String,
    views: u32,
    rating: u32,
    date: String,
    expiration: String,
    likes: u32,
    dislikes: u32,
    format: String,
    size: String,
    comments: Vec<Comment>,
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

async fn post(Form(data): Form<Post>) -> Response {
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap();
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
    let paste_id = response
        .headers()
        .get("Location")
        .unwrap()
        .to_str()
        .unwrap()
        .split("/")
        .last()
        .unwrap();

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

async fn get_icon(client: &reqwest::Client, url: &str) -> String {
    let icon_data = client.get(url).send().await.unwrap().bytes().await.unwrap();
    format!("data:image/jpg;base64,{}", base64::encode(icon_data))
}

async fn get_paste(client: &reqwest::Client, id: &str) -> Paste {
    let paste_response = client.get(URL.to_owned() + "/" + id).send().await.unwrap();
    let body = paste_response.text().await.unwrap();
    let dom = tl::parse(body.as_str(), tl::ParserOptions::default()).unwrap();
    let parser = dom.parser();

    let username = dom.query_selector(".post-view>.details .username>a").unwrap().next().unwrap().get(parser).unwrap().as_tag().unwrap().inner_text(parser).into_owned();
    let icon_url = dom.query_selector(".post-view>.details .user-icon>img").unwrap().next().unwrap().get(parser).unwrap().as_tag().unwrap().attributes().get("src").unwrap().unwrap().as_utf8_str().into_owned();
    let icon = get_icon(client, &(URL.to_owned() + icon_url.as_str())).await;

    let author = User {
        username: "a".to_owned(),
        icon: "a".to_owned(),
    };

    let unlisted = dom.query_selector(".unlisted").is_some();
    println!("{:?}", unlisted);
    let content = dom
        .query_selector(".-raw")
        .unwrap()
        .next()
        .unwrap()
        .get(parser)
        .unwrap()
        .as_tag()
        .unwrap()
        .inner_text(parser)
        .into_owned();
    let title = dom
        .query_selector("h1")
        .unwrap()
        .next()
        .unwrap()
        .get(parser)
        .unwrap()
        .as_tag()
        .unwrap()
        .inner_text(parser)
        .into_owned();
    let views = dom
        .query_selector(".visits")
        .unwrap()
        .next()
        .unwrap()
        .get(parser)
        .unwrap()
        .as_tag()
        .unwrap()
        .inner_text(parser)
        .into_owned()
        .parse()
        .unwrap();
    let rating = dom
        .query_selector(".rating")
        .unwrap()
        .next()
        .unwrap()
        .get(parser)
        .unwrap()
        .as_tag()
        .unwrap()
        .inner_text(parser)
        .into_owned()
        .parse()
        .unwrap();
    let date = dom
        .query_selector(".date")
        .unwrap()
        .next()
        .unwrap()
        .get(parser)
        .unwrap()
        .as_tag()
        .unwrap()
        .attributes()
        .get("title")
        .unwrap()
        .unwrap()
        .as_utf8_str()
        .into_owned();
    let expiration = dom
        .query_selector(".expire")
        .unwrap()
        .next()
        .unwrap()
        .get(parser)
        .unwrap()
        .as_tag()
        .unwrap()
        .inner_text(parser)
        .into_owned();
    let likes = dom
        .query_selector(".-like")
        .unwrap()
        .next()
        .unwrap()
        .get(parser)
        .unwrap()
        .as_tag()
        .unwrap()
        .inner_text(parser)
        .into_owned()
        .parse()
        .unwrap();
    let dislikes = dom
        .query_selector(".-dislike")
        .unwrap()
        .next()
        .unwrap()
        .get(parser)
        .unwrap()
        .as_tag()
        .unwrap()
        .inner_text(parser)
        .into_owned()
        .parse()
        .unwrap();
    let format = dom
        .query_selector("a.btn.-small.h_800")
        .unwrap()
        .next()
        .unwrap()
        .get(parser)
        .unwrap()
        .as_tag()
        .unwrap()
        .inner_text(parser)
        .into_owned();
    let size = dom
        .query_selector(".left")
        .unwrap()
        .next()
        .unwrap()
        .get(parser)
        .unwrap()
        .as_tag()
        .unwrap()
        .inner_text(parser)
        .into_owned();

    Paste {
        author,
        content,
        unlisted,
        title,
        views,
        rating,
        date,
        likes,
        dislikes,
        expiration,
        format,
        size,
        comments: vec![],
    }
}

async fn view(Path(id): Path<String>) -> Response {
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap();
    let paste = get_paste(&client, &id).await;

    // get all the data

    Response::builder()
        .status(200)
        .header("Content-Type", "text/html")
        .body(Body::new(
            TEMPLATES
                .render(
                    "view.html",
                    &tera::Context::from_value(json!({ "content": paste.content })).unwrap(),
                )
                .unwrap(),
        ))
        .unwrap()
}
