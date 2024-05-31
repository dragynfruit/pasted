use crate::{
    client::Client,
    constants::URL,
    paste::{self, Paste},
    templates::TEMPLATES,
};
use axum::{
    body::Body,
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing, Form, Json, Router,
};
use serde::{Deserialize, Serialize};
use tera::Context;
use ureq_multipart::MultipartBuilder;

#[derive(Serialize)]
struct Page {
    id: String,
}

#[derive(Serialize)]
struct LockScreen {
    id: String,
    burn: bool,
}

#[derive(Deserialize)]
struct Unlock {
    password: Option<String>,
}

pub fn get_router(client: Client) -> Router {
    Router::new()
        .route("/raw/:id", routing::get(view_raw))
        .route("/json/:id", routing::get(view_json))
        .route("/dl/:id", routing::get(view_download))
        .route("/print/:id", routing::get(view_print))
        .route("/clone/:id", routing::get(view_clone))
        .route("/embed/:id", routing::get(view_embed))
        .route("/embed_js/:id", routing::get(view_embed_js))
        .route("/embed_iframe/:id", routing::get(view_embed_iframe))
        .route("/:id", routing::get(view).post(view_locked))
        .with_state(client)
}

async fn view_raw(State(client): State<Client>, Path(id): Path<String>) -> impl IntoResponse {
    let content = client.get_string(format!("{URL}/raw/{id}").as_str());

    Response::builder()
        .status(200)
        .header("Content-Type", "text/plain")
        .body(Body::from(content))
        .unwrap()
}

async fn view_json(State(client): State<Client>, Path(id): Path<String>) -> Json<Paste> {
    let dom = client.get_html(format!("{URL}/{id}").as_str());
    let paste = paste::parse_paste(&dom);

    Json(paste)
}

async fn view_download(State(client): State<Client>, Path(id): Path<String>) -> impl IntoResponse {
    let content = client.get_string(format!("{URL}/raw/{id}").as_str());

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

async fn view_print(State(client): State<Client>, Path(id): Path<String>) -> impl IntoResponse {
    let dom = client.get_html(format!("{URL}/{id}").as_str());
    let paste = paste::parse_paste(&dom);

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

async fn view_clone(State(client): State<Client>, Path(id): Path<String>) -> impl IntoResponse {
    let dom = client.get_html(format!("{URL}/{id}").as_str());
    let paste = paste::parse_paste(&dom);

    Response::builder()
        .status(200)
        .header("Content-Type", "text/html")
        .body(Body::new(
            TEMPLATES
                .render("post.html", &Context::from_serialize(paste).unwrap())
                .unwrap(),
        ))
        .unwrap()
}

async fn view_embed(Path(id): Path<String>) -> impl IntoResponse {
    Response::builder()
        .status(200)
        .header("Content-Type", "text/html")
        .body(Body::from(
            TEMPLATES
                .render("embed.html", &Context::from_serialize(Page { id }).unwrap())
                .unwrap(),
        ))
        .unwrap()
}

async fn view_embed_js(State(client): State<Client>, Path(id): Path<String>) -> impl IntoResponse {
    let dom = client.get_html(format!("{URL}/{id}").as_str());
    let paste = paste::parse_paste(&dom);

    Response::builder()
        .status(200)
        .header("Content-Type", "text/javascript")
        .body(Body::from(format!(
            "document.write('{}');",
            TEMPLATES
                .render(
                    "embed_iframe.html",
                    &Context::from_serialize(paste).unwrap()
                )
                .unwrap()
        )))
        .unwrap()
}

async fn view_embed_iframe(
    State(client): State<Client>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let dom = client.get_html(format!("{URL}/{id}").as_str());
    let paste = paste::parse_paste(&dom);

    Response::builder()
        .status(200)
        .header("Content-Type", "text/html")
        .body(Body::from(
            TEMPLATES
                .render(
                    "embed_iframe.html",
                    &Context::from_serialize(paste).unwrap(),
                )
                .unwrap(),
        ))
        .unwrap()
}

async fn view_locked(
    State(client): State<Client>,
    Path(id): Path<String>,
    Form(data): Form<Unlock>,
) -> impl IntoResponse {
    let csrf = paste::get_csrftoken(&client.get_html(format!("{URL}/{id}").as_str()));

    let form = MultipartBuilder::new()
        .add_text("_csrf-frontend", &csrf)
        .unwrap()
        .add_text("PostPasswordVerificationForm[password]", &data.password.unwrap_or("".to_owned()))
        .unwrap()
        .add_text("is_burn", "1")
        .unwrap()
        .finish()
        .unwrap();

    let dom = client.post_html(format!("{URL}/{id}").as_str(), form);
    let paste = paste::parse_paste(&dom);

    Response::builder()
        .status(200)
        .header("Content-Type", "text/html")
        .body(Body::from(
            TEMPLATES
                .render("view.html", &Context::from_serialize(paste).unwrap())
                .unwrap(),
        ))
        .unwrap()
}

async fn view(State(client): State<Client>, Path(id): Path<String>) -> impl IntoResponse {
    let dom = client.get_html(format!("{URL}/{id}").as_str());
    if paste::is_locked(&dom) {
        return Response::builder()
            .status(200)
            .header("Content-Type", "text/html")
            .body(Body::from(
                TEMPLATES
                    .render(
                        "locked.html",
                        &Context::from_serialize(LockScreen {
                            id,
                            burn: paste::is_burn(&dom),
                        })
                        .unwrap(),
                    )
                    .unwrap(),
            ))
            .unwrap();
    } else if paste::is_burn(&dom) {
        return Response::builder()
            .status(200)
            .header("Content-Type", "text/html")
            .body(Body::from(
                TEMPLATES
                    .render(
                        "burn.html",
                        &Context::from_serialize(Page {
                            id
                        })
                        .unwrap(),
                    )
                    .unwrap(),
            ))
            .unwrap();
    }

    let paste = paste::parse_paste(&dom);
    Response::builder()
        .status(200)
        .header("Content-Type", "text/html")
        .body(Body::from(
            TEMPLATES
                .render("view.html", &Context::from_serialize(paste).unwrap())
                .unwrap(),
        ))
        .unwrap()
}
