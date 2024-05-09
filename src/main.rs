use axum::{
    body::Body,
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing, Form, Json, Router,
};
use once_cell::sync::Lazy;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::{env, process, time};
use tera::{Context, Tera};
use ureq::{Agent, AgentBuilder};
use ureq_multipart::MultipartBuilder;

mod routes;
mod constants;
mod request;
mod tools;
mod templates;

#[derive(Deserialize)]
struct Post {
    text: String,
    category: u8,
    tags: String,
    format: u16,
    expiration: String,
    exposure: u8,
    password: String,
    name: String,
}

#[derive(Deserialize, Serialize)]
struct BasicUser {
    username: String,
    registered: bool,
    icon_url: String,
}

#[derive(Deserialize, Serialize)]
struct User {
    user: BasicUser,
    pastes: Vec<PasteInfo>,
    views: u64,
    rating: f32,
    date: String,
    website: String,
}

#[derive(Deserialize, Serialize)]
struct BasicPasteInfo {
    title: String,
    date: String,
    format: String,
}

#[derive(Deserialize, Serialize)]
struct PasteInfo {
    basic_info: BasicPasteInfo,
    views: u64,
    expiration: String,
    num_comments: u64,
    category: String,
}

#[derive(Deserialize, Serialize)]
struct PasteData {
    author: BasicUser,
    content: String,
    likes: u32,
    dislikes: u32,
    size: String,
}

#[derive(Deserialize, Serialize)]
struct Comment {
    data: PasteData,
    date: String,
}

#[derive(Deserialize, Serialize)]
struct Paste {
    info: PasteInfo,
    data: PasteData,
    unlisted: bool,
    rating: f32,
    id: String,
    is_comment: bool,
    comments: Vec<Comment>,
    tags: Vec<String>,
}

#[tokio::main]
async fn main() {
    let port = env::var("PORT").unwrap_or("3000".to_string());
    let host = env::var("HOST").unwrap_or("0.0.0.0".to_string());
    let addr = format!("{}:{}", host, port);

    let agent = AgentBuilder::new()
        .redirects(0)
        .build();

    let app = Router::new()
        .route("/", routing::get(index).post(post))
        .route("/favicon.ico", routing::get(favicon))
        .with_state(agent)
        .merge(routes::get_router());
 
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

/*
    Images
*/

async fn favicon() -> impl IntoResponse {
    Response::builder()
        .status(200)
        .header("Content-Type", "image/x-icon")
        .header("Cache-Control", "public, max-age=31536000, immutable")
        .body(Body::from(include_bytes!("assets/favicon.ico").to_vec()))
        .unwrap()
}

async fn guest() -> impl IntoResponse {
    Response::builder()
        .status(200)
        .header("Content-Type", "image/png")
        .header("Cache-Control", "public, max-age=31536000, immutable")
        .body(Body::from(include_bytes!("assets/guest.png").to_vec()))
        .unwrap()
}

/*
    Make Post
*/

async fn post(State(agent): State<Agent>, Form(data): Form<Post>) -> impl IntoResponse {
    let csrf = get_csrftoken(&agent);

    let form = MultipartBuilder::new()
        .add_text("_csrf-frontend", &csrf)
        .unwrap()
        .add_text("PostForm[text]", &data.text)
        .unwrap()
        .add_text("PostForm[category_id]", &data.category.to_string())
        .unwrap()
        .add_text("PostForm[tag]", &data.tags)
        .unwrap()
        .add_text("PostForm[format]", &data.format.to_string())
        .unwrap()
        .add_text("PostForm[expiration]", &data.expiration.to_string())
        .unwrap()
        .add_text("PostForm[status]", &data.exposure.to_string())
        .unwrap()
        .add_text(
            "PostForm[is_password_enabled]",
            if data.password.is_empty() { "0" } else { "1" },
        )
        .unwrap()
        .add_text("PostForm[password]", &data.password)
        .unwrap()
        .add_text(
            "PostForm[is_burn]",
            if data.expiration == "B" { "1" } else { "0" },
        )
        .unwrap()
        .add_text("PostForm[name]", &data.name)
        .unwrap()
        .add_text("PostForm[is_guest]", "1")
        .unwrap()
        .finish()
        .unwrap();

    let response = agent
        .post(format!("{URL}/").as_str())
        .set("Content-Type", &form.0)
        .send_bytes(&form.1)
        .unwrap();
    let paste_id = response
        .header("Location")
        .unwrap()
        .split("/")
        .last()
        .unwrap();

    Response::builder()
        .status(response.status())
        .header("Location", format!("/{paste_id}"))
        .header("Content-Type", "text/html")
        .body(Body::new(
            TEMPLATES.render("index.html", &Context::new()).unwrap(),
        ))
        .unwrap()
}

async fn index() -> impl IntoResponse {
    Response::builder()
        .status(200)
        .header("Content-Type", "text/html")
        .body(Body::new(
            TEMPLATES.render("index.html", &Context::new()).unwrap(),
        ))
        .unwrap()
}

/*
    User Icons
*/

async fn icon(
    Path((id0, id1, id2, id3)): Path<(String, String, String, String)>,
) -> impl IntoResponse {
    let agent = AgentBuilder::new()
        .redirects(0)
        .build();

    let id3 = id3.split_once(".").unwrap().0;
    let icon = get_bytes(
        &agent,
        format!("{URL}/cache/img/{id0}/{id1}/{id2}/{id3}.jpg").as_str(),
    );

    Response::builder()
        .status(200)
        .header("Content-Type", "image/jpeg")
        .header("Cache-Control", "public, max-age=31536000, immutable")
        .body(Body::from(icon))
        .unwrap()
}

/*
    View Paste
*/

fn get_paste(agent: &Agent, id: &str) -> Paste {
    let dom = get_html(agent, format!("{URL}/{id}").as_str());

    let username = dom
        .select(&Selector::parse(".post-view>.details .username").unwrap())
        .next()
        .unwrap()
        .text()
        .collect::<String>()
        .trim()
        .to_owned();
    let registered = dom
        .select(&Selector::parse(".post-view>.details .username>.a").unwrap())
        .next()
        .is_some();
    let icon_url = dom
        .select(&Selector::parse(".post-view>.details .user-icon>img").unwrap())
        .next()
        .unwrap()
        .value()
        .attr("src")
        .unwrap()
        .replace("/themes/pastebin/img/", "")
        .replace("/cache/img/", "")
        .to_owned();
    let icon_url = format!("/imgs/{}", icon_url);

    let author = BasicUser {
        username,
        registered,
        icon_url,
    };

    let title = dom
        .select(&Selector::parse(".post-view>.details h1").unwrap())
        .next()
        .unwrap()
        .text()
        .collect::<String>();

    let date = dom
        .select(&Selector::parse(".post-view>.details .date").unwrap())
        .next()
        .unwrap()
        .text()
        .collect::<String>()
        .trim()
        .to_owned();

    let format = dom
        .select(&Selector::parse(".post-view>.highlighted-code a.btn.-small.h_800").unwrap())
        .next()
        .unwrap()
        .text()
        .collect::<String>()
        .trim()
        .to_owned();

    let basic_info = BasicPasteInfo {
        title,
        date,
        format,
    };

    let views = dom
        .select(&Selector::parse(".post-view>.details .visits").unwrap())
        .next()
        .unwrap()
        .text()
        .collect::<String>()
        .trim()
        .parse()
        .unwrap();
    let expiration = dom
        .select(&Selector::parse(".post-view>.details .expire").unwrap())
        .next()
        .unwrap()
        .text()
        .collect::<String>()
        .trim()
        .to_owned();
    let category = dom
        .select(&Selector::parse(".post-view>.highlighted-code .left > span:nth-child(2)").unwrap())
        .next()
        .unwrap()
        .text()
        .collect::<String>()
        .trim()
        .split_once(" ")
        .unwrap()
        .1
        .to_owned();

    let info = PasteInfo {
        basic_info,
        views,
        expiration,
        num_comments: 0,
        category,
    };

    let content = dom
        .select(&Selector::parse(".post-view>.highlighted-code .source > ol").unwrap())
        .next()
        .unwrap()
        .text()
        .collect::<String>()
        .to_owned();
    let likes = dom
        .select(&Selector::parse(".post-view>.highlighted-code .-like").unwrap())
        .next()
        .unwrap()
        .text()
        .collect::<String>()
        .trim()
        .parse()
        .unwrap();
    let dislikes = dom
        .select(&Selector::parse(".post-view>.highlighted-code .-dislike").unwrap())
        .next()
        .unwrap()
        .text()
        .collect::<String>()
        .trim()
        .parse()
        .unwrap();
    let size = dom
        .select(&Selector::parse(".post-view>.highlighted-code .left").unwrap())
        .next()
        .unwrap()
        .text()
        .collect::<String>()
        .trim()
        .split_once(" ")
        .unwrap()
        .1
        .split_once("\n")
        .unwrap()
        .0
        .to_owned();

    let data = PasteData {
        author,
        content,
        likes,
        dislikes,
        size,
    };

    let unlisted = dom
        .select(&Selector::parse(".post-view>.details .unlisted").unwrap())
        .next()
        .is_some();
    let rating = dom
        .select(&Selector::parse(".post-view>.details .rating").unwrap())
        .next()
        .unwrap()
        .text()
        .collect::<String>()
        .trim()
        .parse()
        .unwrap();
    let tags = dom
        .select(&Selector::parse(".post-view>.tags > a").unwrap())
        .map(|el| el.text().collect::<String>().to_owned())
        .collect::<Vec<String>>();

    Paste {
        info,
        data,
        unlisted,
        rating,
        id: id.to_owned(),
        is_comment: false,
        comments: vec![],
        tags,
    }
}

async fn view_raw(State(agent): State<Agent>, Path(id): Path<String>) -> impl IntoResponse {
    let content = get_body(&agent, format!("{URL}/raw/{id}").as_str());

    Response::builder()
        .status(200)
        .header("Content-Type", "text/plain")
        .body(Body::from(content))
        .unwrap()
}

async fn view_json(State(agent): State<Agent>, Path(id): Path<String>) -> Json<Paste> {
    let paste = get_paste(&agent, &id);

    Json(paste)
}

async fn view_download(State(agent): State<Agent>, Path(id): Path<String>) -> impl IntoResponse {
    let content = get_body(&agent, format!("{URL}/raw/{id}").as_str());

    Response::builder()
        .status(200)
        .header("Content-Type", "text/plain")
        .header(
            "Content-Disposition",
            format!("attachment; filename=\"{id}.txt\""),
        )
        .body(Body::from(content))
        .unwrap()
}

async fn view_print(State(agent): State<Agent>, Path(id): Path<String>) -> impl IntoResponse {
    let paste = get_paste(&agent, &id);

    Response::builder()
        .status(200)
        .header("Content-Type", "text/html")
        .body(Body::from(
            TEMPLATES
                .render("print.html", &Context::from_serialize(paste).unwrap())
                .unwrap(),
        ))
        .unwrap()
}

async fn view(State(agent): State<Agent>, Path(id): Path<String>) -> impl IntoResponse {
    let paste = get_paste(&agent, &id);

    Response::builder()
        .status(200)
        .header("Content-Type", "text/html")
        .body(Body::from(
            TEMPLATES
                .render("view.html", &Context::from_serialize(paste).unwrap())
                .unwrap(),
        ))
        .unwrap()
        .into_response()
}
