use axum::{Router, routing::{get, post}, Extension, http::StatusCode, response::IntoResponse, Json};
use surrealdb::{Surreal, engine::remote::ws::Client};
use crate::model::DoodleEntry;
use anyhow::Result;
use minijinja::render;

async fn recent_doodles(db : Extension<Surreal<Client>>) -> axum::response::Html<String>
{
    let doodles : Vec<DoodleEntry> = db
        .select("Doodles")
        .await
        .expect("Failed to load doodles");
    //TODO: Add error-handling instead of expect

    let resp = render!("
        <h1> Doodles </h1>
        {% for doodle in doodles %}
            <div>
                <h2>{{ doodle.name }}</h2>
                <p>{{ doodle.description }}</p>
            </div>
        {% endfor %}
    ",
    doodles);
    resp.into()

}

async fn create_doodle(db : Extension<Surreal<Client>>,Json(payload): Json<DoodleEntry>) -> StatusCode
{
    let doodle = DoodleEntry {
        name: payload.name,
        description: payload.description,
    };
    let x : Result<Vec<DoodleEntry>,surrealdb::Error> = db.create("Doodles").content::<DoodleEntry>(doodle).await;
    if !x.is_ok()
    {
        println!("Failed to create doodle: {:?}",x.err());
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    StatusCode::CREATED
}

pub fn create_doodle_service(db : Surreal<Client>) -> Router
{
    Router::new()
        .route("/recent-doodles",get(recent_doodles))
        .route("/create-doodle", post(create_doodle))
        .layer(Extension(db))

}