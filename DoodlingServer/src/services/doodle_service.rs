use axum::{Router, routing::get, Extension};
use surrealdb::{Surreal, engine::remote::ws::Client};


async fn recent_doodles(db : Extension<Surreal<Client>>) -> String
{
    db.version().await.unwrap().to_string()
}

pub fn create_doodle_service(db : Surreal<Client>) -> Router
{
    Router::new()
        .route("/recent-doodles",get(recent_doodles))
        .layer(Extension(db))

}