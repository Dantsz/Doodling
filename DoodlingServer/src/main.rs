use anyhow::Ok;
use axum::{
    routing::{get, post},
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_http::services::{ServeDir, ServeFile};
#[tokio::main]
async fn main() -> anyhow::Result<()>{
    println!("Starting server...");
    println!("Running on {}:{}",dotenv::var("DOODLING_HOST").unwrap(),dotenv::var("DOODLING_PORT").unwrap());
    let addr :SocketAddr = format!("{}:{}",dotenv::var("DOODLING_HOST").unwrap(),dotenv::var("DOODLING_PORT").unwrap()).parse()?;

    let serve_dir = ServeDir::new("../DoolingHtmx");
    let app = Router::new()
        .route("/", get(root))
        .fallback_service(serve_dir);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
    Ok(())
}async fn root() -> &'static str {
    "Hello, World!"
}

