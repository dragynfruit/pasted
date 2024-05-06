use axum::{
    body::Body,
    extract::Path,
    response::{IntoResponse, Response},
    routing, Form, Json, Router,
};
use cookie_store::CookieStore;
use once_cell::sync::Lazy;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::{env, process, time};
use tera::{Context, Tera};
use ureq::AgentBuilder;
use ureq_multipart::MultipartBuilder;

const URL: &str = "https://pastebin.com";

static TEMPLATES: Lazy<Tera> = Lazy::new(|| {
    let mut tera = match Tera::new("templates/*") {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            process::exit(1);
        }
    };
    tera.autoescape_on(vec![".html", ".sql"]);
    tera
});

#[derive(Deserialize, Serialize, Clone)]
struct InstanceInfo {
    version: String,
    name: String,
    start_time: String,
    is_release: bool,
}

static INSTANCE_INFO: Lazy<InstanceInfo> = Lazy::new(|| InstanceInfo {
    version: env!("CARGO_PKG_VERSION").to_string(),
    name: env!("CARGO_PKG_NAME").to_string(),
    start_time: time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string(),
    is_release: cfg!(debug_assertions),
});

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
    category: String,
    id: String,
    comments: Vec<Comment>,
    tags: Vec<String>,
}

#[tokio::main]
async fn main() {
    let port = env::var("PORT").unwrap_or("3000".to_string());
    let host = env::var("HOST").unwrap_or("0.0.0.0".to_string());
    let addr = format!("{}:{}", host, port);

    let app = Router::new()
        .route("/", routing::get(index).post(post))
        .route("/favicon.ico", routing::get(favicon))
        .route("/info", routing::get(info))
        .route("/info.json", routing::get(info_raw))
        .route("/imgs/guest.png", routing::get(guest))
        .route("/imgs/:id0/:id1/:id2/:id3.jpg", routing::get(icon))
        .route("/raw/:id", routing::get(view_raw))
        .route("/json/:id", routing::get(view_json))
        .route("/dl/:id", routing::get(view_download))
        .route("/print/:id", routing::get(view_print))
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
    Images
*/

async fn favicon() -> impl IntoResponse {
    Response::builder()
        .status(200)
        .header("Content-Type", "image/x-icon")
        .header("Cache-Control", "public, max-age=31536000, immutable")
        .body(Body::from(include_bytes!("favicon.ico").to_vec()))
        .unwrap()
}

async fn guest() -> impl IntoResponse {
    Response::builder()
        .status(200)
        .header("Content-Type", "image/png")
        .header("Cache-Control", "public, max-age=31536000, immutable")
        .body(Body::from(include_bytes!("guest.png").to_vec()))
        .unwrap()
}

/*
    Info
*/

async fn info() -> impl IntoResponse {
    Response::builder()
        .status(200)
        .header("Content-Type", "text/html")
        .body(Body::new(
            TEMPLATES
                .render(
                    "info.html",
                    &Context::from_serialize(&*INSTANCE_INFO).unwrap(),
                )
                .unwrap(),
        ))
        .unwrap()
}

async fn info_raw() -> Json<InstanceInfo> {
    Json(INSTANCE_INFO.clone())
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
        .redirects(0)
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

fn get_bytes(agent: &ureq::Agent, url: &str) -> Vec<u8> {
    let mut data = Vec::new();
    agent
        .get(url)
        .call()
        .unwrap()
        .into_reader()
        .read_to_end(&mut data)
        .unwrap();
    data
}

async fn icon(
    Path((id0, id1, id2, id3)): Path<(String, String, String, String)>,
) -> impl IntoResponse {
    let agent = AgentBuilder::new()
        .cookie_store(CookieStore::default())
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

fn get_paste(agent: &ureq::Agent, id: &str) -> Paste {
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

    let info = PasteInfo {
        basic_info,
        views,
        expiration,
        num_comments: 0,
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
    let tags = dom
        .select(&Selector::parse(".post-view>.tags > a").unwrap())
        .map(|el| el.text().collect::<String>().to_owned())
        .collect::<Vec<String>>();

    Paste {
        info,
        data,
        unlisted,
        rating,
        category,
        id: id.to_owned(),
        comments: vec![],
        tags,
    }
}

async fn view_raw(Path(id): Path<String>) -> impl IntoResponse {
    let agent = AgentBuilder::new()
        .cookie_store(CookieStore::default())
        .redirects(0)
        .build();
    let content = get_body(&agent, format!("{URL}/raw/{id}").as_str());

    Response::builder()
        .status(200)
        .header("Content-Type", "text/plain")
        .body(Body::from(content))
        .unwrap()
}

async fn view_json(Path(id): Path<String>) -> Json<Paste> {
    let agent = AgentBuilder::new()
        .cookie_store(CookieStore::default())
        .redirects(0)
        .build();
    let paste = get_paste(&agent, &id);

    Json(paste)
}

async fn view_download(Path(id): Path<String>) -> impl IntoResponse {
    let agent = AgentBuilder::new()
        .cookie_store(CookieStore::default())
        .redirects(0)
        .build();
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

async fn view_print(Path(id): Path<String>) -> impl IntoResponse {
    let agent = AgentBuilder::new()
        .cookie_store(CookieStore::default())
        .redirects(0)
        .build();
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

async fn view(Path(id): Path<String>) -> impl IntoResponse {
    let agent = AgentBuilder::new()
        .cookie_store(CookieStore::default())
        .redirects(0)
        .build();
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
