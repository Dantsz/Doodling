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
async fn main() {
    let serve_dir = ServeDir::new("../DoolingHtmx");
    let app = Router::new()
        .route("/", get(root))
        .fallback_service(serve_dir);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root() -> &'static str {
    "Hello, World!"
}

