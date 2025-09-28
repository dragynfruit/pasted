use axum;
use state::AppState;
use std::env;
use tokio::net::TcpListener;

mod client;
mod constants;
mod parsers;
mod routes;
mod state;
mod templates;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port = env::var("PORT").unwrap_or("3000".to_string());
    let host = env::var("HOST").unwrap_or("0.0.0.0".to_string());
    let addr = format!("{}:{}", host, port);

    routes::info::DEPLOY_DATE.get_or_init(|| chrono::Local::now().to_rfc2822());

    let state = match AppState::try_default() {
        Ok(state) => state,
        Err(e) => {
            eprintln!("Failed to initialize application state: {}", e);
            return Err(e.into());
        }
    };

    let app = routes::get_router(state);

    let listener = match TcpListener::bind(&addr).await {
        Ok(listener) => listener,
        Err(e) => {
            eprintln!("Failed to bind to address {}: {}", addr, e);
            return Err(e.into());
        }
    };

    println!("Listening at {}", addr);

    if let Err(e) = axum::serve(listener, app)
        .with_graceful_shutdown(async {
            match tokio::signal::ctrl_c().await {
                Ok(_) => println!("Received shutdown signal"),
                Err(e) => eprintln!("Failed to install CTRL+C signal handler: {}", e),
            }
        })
        .await
    {
        eprintln!("Server error: {}", e);
        return Err(e.into());
    }

    println!("Shutting down");
    Ok(())
}
