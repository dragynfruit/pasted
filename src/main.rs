
use axum;
use state::AppState;
use std::env;
use tokio::net::TcpListener;

mod client;
mod constants;
mod paste;
mod state;
mod routes;
mod templates;

#[tokio::main]
async fn main() {
    let port = env::var("PORT").unwrap_or("3000".to_string());
    let host = env::var("HOST").unwrap_or("0.0.0.0".to_string());
    let addr = format!("{}:{}", host, port);

    let state = AppState::default();
    let app = routes::get_router(state);
    let listener = TcpListener::bind(&addr).await.unwrap();

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