use crate::client::Client;
use axum::{
    body::Body,
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use scraper::Selector;
use serde::{Deserialize, Serialize};
use std::env;
use tera::Context;
use ureq::Agent;

mod client;
mod constants;
mod paste;
mod routes;
mod paste;

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

    let client = Client::new();

    let app = routes::get_router(client);
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
