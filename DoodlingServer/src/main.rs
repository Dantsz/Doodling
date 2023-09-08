mod services;
mod model;
mod templates;
mod middleware;
use anyhow::Ok;
use axum::Router;
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use surrealdb::engine::remote::ws::Ws;
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use log::{info,trace};

use crate::{services::doodle_service, middleware::database_layer::SurrealDoodleConnection};
#[tokio::main]
async fn main() -> anyhow::Result<()>{

    let crate_name = env!("CARGO_PKG_NAME");
    env_logger::Builder::new().filter_module(crate_name, log::LevelFilter::Trace).init();

    info!("Starting server...");
    let addr :SocketAddr = format!("{}:{}",dotenv::var("DOODLING_HOST").unwrap(),dotenv::var("DOODLING_PORT").unwrap()).parse()?;

    info!("Running on {}:{}",dotenv::var("DOODLING_HOST").unwrap(),dotenv::var("DOODLING_PORT").unwrap());

    trace!("Connecting to database...");
    let connection_string = format!("{}:{}",dotenv::var("DOODLING_DB_HOST").unwrap(),dotenv::var("DOODLING_DB_PORT").unwrap());
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
    let app = Router::new()
        .nest("/api",doodle_service::create_doodle_service(db_con))
        .fallback_service(ServeDir::new("./DoodlingHtmx/resources"));
    info!("Server started");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
    info!("Server stopped");
    Ok(())
}

