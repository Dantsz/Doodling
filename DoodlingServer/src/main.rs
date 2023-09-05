mod services;
mod model;
use anyhow::Ok;
use axum::{
    routing::{get, post},
    http::{StatusCode, Response},
    Router
};
use tower::{ServiceBuilder, ServiceExt};
use std::net::SocketAddr;
use tower_http::services::{ServeDir,fs::ServeFileSystemResponseBody};
use surrealdb::engine::remote::ws::Ws;
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use log::{info,warn,trace};
use http_body::{Body as _, Full};

use crate::services::doodle_service;
#[tokio::main]
async fn main() -> anyhow::Result<()>{

    let crate_name = env!("CARGO_PKG_NAME");
    env_logger::Builder::new().filter_module(crate_name, log::LevelFilter::Trace).init();

    info!("Starting server...");
    let addr :SocketAddr = format!("{}:{}",dotenv::var("DOODLING_HOST").unwrap(),dotenv::var("DOODLING_PORT").unwrap()).parse()?;

    info!("Running on {}:{}",dotenv::var("DOODLING_HOST").unwrap(),dotenv::var("DOODLING_PORT").unwrap());

    trace!("Connecting to database...");
    let db = Surreal::new::<Ws>(format!("{}:{}",dotenv::var("DOODLING_DB_HOST").unwrap(),dotenv::var("DOODLING_DB_PORT").unwrap())).await?;
    db.signin(Root {
        username: &dotenv::var("DOODLING_DB_USER").unwrap(),
        password: &dotenv::var("DOODLING_DB_PASSWORD").unwrap(),
    })
    .await?;
    trace!("Setting namespace...");
    db.use_ns("a").use_db("a").await?;

    trace!("Creating app...");
    let app = Router::new()
        .nest("/api",doodle_service::create_doodle_service(db))
        .fallback_service(ServeDir::new("./DoodlingHtmx"));
    info!("Server started");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

