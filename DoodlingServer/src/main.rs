mod services;
mod model;
use anyhow::Ok;
use axum::{
    routing::{get, post},
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};
use env_logger::Env;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_http::services::{ServeDir, ServeFile};
use surrealdb::engine::remote::ws::Ws;
use surrealdb::opt::auth::Root;
use surrealdb::sql::Thing;
use surrealdb::Surreal;
use log::{info, warn,trace};

use crate::services::doodle_service;
#[tokio::main]
async fn main() -> anyhow::Result<()>{

    let crate_name = env!("CARGO_PKG_NAME");
    env_logger::Builder::new().filter_module(crate_name, log::LevelFilter::Trace).init();

    trace!("Starting server...");
    trace!("Running on {}:{}",dotenv::var("DOODLING_HOST").unwrap(),dotenv::var("DOODLING_PORT").unwrap());
    let addr :SocketAddr = format!("{}:{}",dotenv::var("DOODLING_HOST").unwrap(),dotenv::var("DOODLING_PORT").unwrap()).parse()?;

    let db = Surreal::new::<Ws>(format!("{}:{}",dotenv::var("DOODLING_DB_HOST").unwrap(),dotenv::var("DOODLING_DB_PORT").unwrap())).await?;
    db.signin(Root {
        username: &dotenv::var("DOODLING_DB_USER").unwrap(),
        password: &dotenv::var("DOODLING_DB_PASSWORD").unwrap(),
    })
    .await?;

    db.use_ns("a").use_db("a").await?;
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

