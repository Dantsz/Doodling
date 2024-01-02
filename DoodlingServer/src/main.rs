mod middleware;
mod model;
mod services;
mod templates;
use anyhow::Ok;
use axum::{
    handler::HandlerWithoutStateExt,
    http::StatusCode,
    response::IntoResponse,
};
use log::{info, trace};
use std::net::SocketAddr;
use surrealdb::engine::remote::ws::Ws;
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use tower_http::services::ServeDir;

use crate::{middleware::database_layer::SurrealDoodleConnection, services::doodle_service};
async fn not_found_handler() -> impl IntoResponse {
    info!("Not Found");
    StatusCode::NOT_FOUND
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let crate_name = env!("CARGO_PKG_NAME");
    env_logger::Builder::new()
        .filter_module(crate_name, log::LevelFilter::Trace)
        .init();

    info!("Starting server...");
    let addr: SocketAddr = format!(
        "{}:{}",
        dotenv::var("DOODLING_HOST").unwrap(),
        dotenv::var("DOODLING_PORT").unwrap()
    )
    .parse()?;

    info!(
        "Running on {}:{}",
        dotenv::var("DOODLING_HOST").unwrap(),
        dotenv::var("DOODLING_PORT").unwrap()
    );

    trace!("Connecting to database...");
    let connection_string = format!(
        "{}:{}",
        dotenv::var("DOODLING_DB_HOST").unwrap(),
        dotenv::var("DOODLING_DB_PORT").unwrap()
    );
    let db = Surreal::new::<Ws>(connection_string).await?;
    db.signin(Root {
        username: &dotenv::var("DOODLING_DB_USER").unwrap(),
        password: &dotenv::var("DOODLING_DB_PASSWORD").unwrap(),
    })
    .await?;
    trace!("Setting namespace...");
    db.use_ns("a").use_db("a").await?;

    let db_con = SurrealDoodleConnection::new(db).await;

    trace!("Creating app...");
    let dir = ServeDir::new("./DoodlingHtmx/resources")
        .not_found_service(not_found_handler.into_service());
    let app = Router::new()
        .nest("/api", doodle_service::create_doodle_service(db_con))
        .nest_service("/", dir)
        .fallback(not_found_handler);

    info!("Configuring server...");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("Server started");
    axum::serve(listener, app.into_make_service()).await?;
    info!("Server stopped");
    Ok(())
}
