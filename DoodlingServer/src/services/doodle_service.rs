use axum::{Router, routing::{get, post}, Extension, http::{StatusCode, HeaderMap, header}, response::IntoResponse, Json};
use surrealdb::{Surreal, engine::remote::ws::Client};
use crate::{model::DoodleEntry, include_template};
use anyhow::Result;
use minijinja::render;
use log::{trace,error};

async fn recent_doodles(db : Extension<Surreal<Client>>) -> impl IntoResponse
{
    trace!("Serving recent doodles");
    let doodles : Vec<DoodleEntry> = db
        .select("Doodles")
        .await
        .expect("Failed to load doodles");
    //TODO: Add error-handling instead of expect

    let resp = render!(include_template!{"doodle_list"}, doodles);

    ([(header::CONTENT_TYPE,"text/html")],resp)
}
async fn create_doodle(db : Extension<Surreal<Client>>,Json(payload): Json<DoodleEntry>) -> impl IntoResponse
{
    trace!("Creating doodle: {}",payload.name);
    let doodle = DoodleEntry {
        name: payload.name,
        description: payload.description,
        data : payload.data
    };
    let x : Result<Vec<DoodleEntry>,surrealdb::Error> = db.create("Doodles").content::<DoodleEntry>(doodle).await;
    let mut header = HeaderMap::new();
    if !x.is_ok()
    {
        error!("Failed to create doodle: {:?}",x.err());
        return (StatusCode::INTERNAL_SERVER_ERROR,header);
    }
    header.insert("HX-Redirect",format!("/index.html").parse().unwrap());
    (StatusCode::CREATED,header)
}

pub fn create_doodle_service(db : Surreal<Client>) -> Router
{
    Router::new()
        .route("/recent-doodles",get(recent_doodles))
        .route("/create-doodle", post(create_doodle))
        .layer(Extension(db))
}