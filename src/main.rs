use axum::{
    body::Body,
    extract::Path,
    response::{IntoResponse, Response},
    routing, Form, Router,
};
use base64::{engine, Engine as _};
use cookie_store::CookieStore;
use once_cell::sync::Lazy;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::env;
use ureq::AgentBuilder;
use ureq_multipart::MultipartBuilder;

const URL: &str = "https://pastebin.com";

static TEMPLATES: Lazy<tera::Tera> = Lazy::new(|| {
    let mut tera = match tera::Tera::new("templates/*") {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    };
    tera.autoescape_on(vec![".html", ".sql"]);
    tera
});

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

#[derive(Deserialize, Serialize, Debug)]
struct User {
    username: String,
    icon: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct Comment {
    author: User,
    text: String,
    date: String,
    likes: u32,
    dislikes: u32,
    format: String,
    link: String,
}

#[derive(Deserialize, Serialize, Debug)]
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
    category: String,
    comments: Vec<Comment>,
}

#[tokio::main]
async fn main() {
    let port = env::var("PORT").unwrap_or("3000".to_string());
    let host = env::var("HOST").unwrap_or("0.0.0.0".to_string());
    let addr = format!("{}:{}", host, port);

    let app = Router::new()
        .route("/", routing::get(index).post(post))
        .route("/favicon.ico", routing::get(favicon))
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

/*
    Favicon
*/

async fn favicon() -> impl IntoResponse {
    Response::builder()
        .status(200)
        .header("Content-Type", "image/x-icon")
        .body(Body::from(include_bytes!("favicon.ico").to_vec()))
        .unwrap()
}

/*
    Make Post
*/

fn get_body(agent: &ureq::Agent, url: &str) -> String {
    agent.get(&url).call().unwrap().into_string().unwrap()
}

fn get_html(agent: &ureq::Agent, url: &str) -> Html {
    Html::parse_document(&get_body(agent, url))
}

fn get_csrftoken(agent: &ureq::Agent) -> String {
    let dom = get_html(agent, format!("{}/", URL).as_str());
    let csrf = dom
        .select(&Selector::parse("meta[name=csrf-token]").unwrap())
        .next()
        .unwrap()
        .value()
        .attr("content")
        .unwrap()
        .to_owned();
    csrf
}

async fn post(Form(data): Form<Post>) -> impl IntoResponse {
    let agent = AgentBuilder::new()
        .cookie_store(CookieStore::default())
        .build();
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
            if data.expiration == 'B' { "1" } else { "0" },
        )
        .unwrap()
        .add_text("PostForm[name]", &data.name)
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
            TEMPLATES
                .render("index.html", &tera::Context::new())
                .unwrap(),
        ))
        .unwrap()
}

async fn index() -> impl IntoResponse {
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

/*
    View Paste
*/

fn get_icon(agent: &ureq::Agent, url: &str) -> String {
    let mut icon_data = Vec::new();
    agent
        .get(url)
        .call()
        .unwrap()
        .into_reader()
        .read_to_end(&mut icon_data)
        .unwrap();
    format!(
        "data:image/jpg;base64,{}",
        engine::general_purpose::STANDARD.encode(icon_data)
    )
}

fn get_paste(agent: &ureq::Agent, id: &str) -> Paste {
    let dom = get_html(agent, format!("{URL}/{id}").as_str());

    let username = dom
        .select(&Selector::parse(".post-view>.details .username>a").unwrap())
        .next()
        .unwrap()
        .text()
        .collect::<String>();
    let icon_url = dom
        .select(&Selector::parse(".post-view>.details .user-icon>img").unwrap())
        .next()
        .unwrap()
        .value()
        .attr("src")
        .unwrap()
        .to_owned();
    let icon = get_icon(agent, &(URL.to_owned() + icon_url.as_str()));

    let author = User { username, icon };

    let unlisted = dom
        .select(&Selector::parse(".unlisted").unwrap())
        .next()
        .is_some();
    let title = dom
        .select(&Selector::parse("h1").unwrap())
        .next()
        .unwrap()
        .text()
        .collect::<String>();
    let views = dom
        .select(&Selector::parse(".visits").unwrap())
        .next()
        .unwrap()
        .text()
        .collect::<String>()
        .trim()
        .parse()
        .unwrap();
    let rating = dom
        .select(&Selector::parse(".rating").unwrap())
        .next()
        .unwrap()
        .text()
        .collect::<String>()
        .trim()
        .parse()
        .unwrap();
    let date = dom
        .select(&Selector::parse(".date").unwrap())
        .next()
        .unwrap()
        .text()
        .collect::<String>()
        .trim()
        .to_owned();
    let expiration = dom
        .select(&Selector::parse(".expire").unwrap())
        .next()
        .unwrap()
        .text()
        .collect::<String>()
        .trim()
        .to_owned();
    let likes = dom
        .select(&Selector::parse(".-like").unwrap())
        .next()
        .unwrap()
        .text()
        .collect::<String>()
        .trim()
        .parse()
        .unwrap();
    let dislikes = dom
        .select(&Selector::parse(".-dislike").unwrap())
        .next()
        .unwrap()
        .text()
        .collect::<String>()
        .trim()
        .parse()
        .unwrap();
    let format = dom
        .select(&Selector::parse("a.btn.-small.h_800").unwrap())
        .next()
        .unwrap()
        .text()
        .collect::<String>()
        .trim()
        .to_owned();
    let size = dom
        .select(&Selector::parse(".left").unwrap())
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
    let category = dom
        .select(&Selector::parse(".left > span:nth-child(2)").unwrap())
        .next()
        .unwrap()
        .text()
        .collect::<String>()
        .trim()
        .split_once(" ")
        .unwrap()
        .1
        .to_owned();
    let content = dom
        .select(&Selector::parse(".source > ol").unwrap())
        .next()
        .unwrap()
        .text()
        .collect::<String>()
        .to_owned();

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
        category,
        comments: vec![],
    }
}

async fn view(Path(id): Path<String>) -> impl IntoResponse {
    let agent = AgentBuilder::new()
        .cookie_store(CookieStore::default())
        .build();
    let paste = get_paste(&agent, &id);

    // get all the data
    Response::builder()
        .status(200)
        .header("Content-Type", "text/html")
        .body(Body::from(
            TEMPLATES
                .render("view.html", &tera::Context::from_serialize(paste).unwrap())
                .unwrap(),
        ))
        .unwrap()
        .into_response()
}
