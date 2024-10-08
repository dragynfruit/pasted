use crate::{
    constants::URL,
    parsers::{
        paste::{self, Paste},
        FromHtml as _,
    },
    state::AppState,
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

use super::error;

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

pub fn get_router(state: AppState) -> Router {
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
        .with_state(state)
}

async fn view_raw(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let content = state.client.get_string(format!("{URL}/raw/{id}").as_str());

    match content {
        Ok(content) => Response::builder()
            .status(200)
            .header("Content-Type", "text/plain")
            .body(Body::from(content))
            .unwrap(),
        Err(err) => error::construct_error(err),
    }
}

async fn view_json(State(state): State<AppState>, Path(id): Path<String>) -> Json<Paste> {
    let dom = state.client.get_html(format!("{URL}/{id}").as_str()).unwrap(); // fix
    let paste = Paste::from_html(&dom);

    Json(paste)
}

async fn view_download(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let content = state.client.get_string(format!("{URL}/raw/{id}").as_str());

    match content {
        Ok(content) => Response::builder()
            .status(200)
            .header("Content-Type", "text/plain")
            .header(
                "Content-Disposition",
                format!("attachment; filename=\"{id}.txt\""),
            )
            .body(Body::from(content))
            .unwrap(),
        Err(err) => error::construct_error(err),
    }
}

async fn view_print(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let dom = state.client.get_html(format!("{URL}/{id}").as_str());

    match dom {
        Ok(dom) => Response::builder()
            .status(200)
            .header("Content-Type", "text/html")
            .body(Body::from(
                TEMPLATES
                    .render(
                        "print.html",
                        &Context::from_serialize(Paste::from_html(&dom)).unwrap(),
                    )
                    .unwrap(),
            ))
            .unwrap(),
        Err(err) => error::construct_error(err),
    }
}

async fn view_clone(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let dom = state.client.get_html(format!("{URL}/{id}").as_str());

    match dom {
        Ok(dom) => Response::builder()
            .status(200)
            .header("Content-Type", "text/html")
            .body(Body::from(
                TEMPLATES
                    .render(
                        "post.html",
                        &Context::from_serialize(Paste::from_html(&dom)).unwrap(),
                    )
                    .unwrap(),
            ))
            .unwrap(),
        Err(err) => error::construct_error(err),
    }
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

async fn view_embed_js(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let dom = state.client.get_html(format!("{URL}/{id}").as_str());

    match dom {
        Ok(dom) => Response::builder()
            .status(200)
            .header("Content-Type", "text/javascript")
            .body(Body::from(format!(
                "document.write('{}');",
                TEMPLATES
                    .render(
                        "embed_iframe.html",
                        &Context::from_serialize(Paste::from_html(&dom)).unwrap()
                    )
                    .unwrap()
            )))
            .unwrap(),
        Err(err) => error::construct_error(err),
    }
}

async fn view_embed_iframe(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let dom = state.client.get_html(format!("{URL}/{id}").as_str());

    match dom {
        Ok(dom) => Response::builder()
            .status(200)
            .header("Content-Type", "text/html")
            .body(Body::from(
                TEMPLATES
                    .render(
                        "embed_iframe.html",
                        &Context::from_serialize(Paste::from_html(&dom)).unwrap(),
                    )
                    .unwrap(),
            ))
            .unwrap(),
        Err(err) => error::construct_error(err),
    }
}

async fn view_locked(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Form(data): Form<Unlock>,
) -> impl IntoResponse {
    let csrf = state.client.get_html(format!("{URL}/{id}").as_str());

    if csrf.is_err() {
        return error::construct_error(csrf.err().unwrap());
    }

    let csrf = paste::get_csrftoken(&csrf.unwrap());

    let form = MultipartBuilder::new()
        .add_text("_csrf-frontend", &csrf)
        .unwrap()
        .add_text(
            "PostPasswordVerificationForm[password]",
            &data.password.unwrap_or("".to_owned()),
        )
        .unwrap()
        .add_text("is_burn", "1")
        .unwrap()
        .finish()
        .unwrap();

    let dom = state.client.post_html(format!("{URL}/{id}").as_str(), form);

    match dom {
        Ok(dom) => Response::builder()
            .status(200)
            .header("Content-Type", "text/html")
            .body(Body::from(
                TEMPLATES
                    .render(
                        "view.html",
                        &Context::from_serialize(Paste::from_html(&dom)).unwrap(),
                    )
                    .unwrap(),
            ))
            .unwrap(),
        Err(err) => error::construct_error(err),
    }
}

async fn view(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let dom = state.client.get_html(format!("{URL}/{id}").as_str());

    if dom.is_err() {
        return error::construct_error(dom.err().unwrap());
    }

    let dom = dom.unwrap();

    let rendered = if paste::is_locked(&dom) {
        TEMPLATES
            .render(
                "locked.html",
                &Context::from_serialize(LockScreen {
                    id,
                    burn: paste::is_burn(&dom),
                })
                .unwrap(),
            )
            .unwrap()
    } else if paste::is_burn(&dom) {
        TEMPLATES
            .render("burn.html", &Context::from_serialize(Page { id }).unwrap())
            .unwrap()
    } else {
        let paste = Paste::from_html(&dom);
        TEMPLATES
            .render("view.html", &Context::from_serialize(paste).unwrap())
            .unwrap()
    };

    Response::builder()
        .status(200)
        .header("Content-Type", "text/html")
        .body(Body::from(rendered))
        .unwrap()
}
