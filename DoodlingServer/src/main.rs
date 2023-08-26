mod services;
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
use surrealdb::engine::remote::ws::Ws;
use surrealdb::opt::auth::Root;
use surrealdb::sql::Thing;
use surrealdb::Surreal;

use crate::services::doodle_service;
#[tokio::main]
async fn main() -> anyhow::Result<()>{
    println!("Starting server...");
    println!("Running on {}:{}",dotenv::var("DOODLING_HOST").unwrap(),dotenv::var("DOODLING_PORT").unwrap());
    let addr :SocketAddr = format!("{}:{}",dotenv::var("DOODLING_HOST").unwrap(),dotenv::var("DOODLING_PORT").unwrap()).parse()?;

    let db = Surreal::new::<Ws>("127.0.0.1:8000").await?;
    db.signin(Root {
        username: "root",
        password: "root",
    })
    .await?;

    let serve_dir = ServeDir::new("../DoolingHtmx");
    let app_test : Router = Router::new()
        .route("/test", get(root));


    let app = Router::new()
        .route("/", get(root))
        .merge(doodle_service::create_doodle_service(db))
        .merge(app_test)
        .fallback_service(serve_dir);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}
async fn root() -> &'static str {
    "Hello, World!"
}

