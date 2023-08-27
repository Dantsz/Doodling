use axum::{Router, routing::get, Extension};
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

pub fn create_doodle_service(db : Surreal<Client>) -> Router
{
    Router::new()
        .route("/recent-doodles",get(recent_doodles))
        .layer(Extension(db))

}