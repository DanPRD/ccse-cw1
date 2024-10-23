#![allow(dead_code)]
use axum::{http::StatusCode, response::{Html, IntoResponse}, routing::get, Router};
use ecom::{browse, product};
use tower_http::{services::{ServeDir, ServeFile}, trace::TraceLayer};
use tracing;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use askama::Template;
use diesel_async::{pooled_connection::{deadpool::Pool, AsyncDieselConnectionManager}, AsyncMysqlConnection};
use dotenvy::dotenv;
use std::env;

mod db;
mod auth;
mod ecom;
use auth::{signin::{process_sign_in, sign_in}, signup::{process_sign_up, sign_up}};

#[derive(Clone)]
struct AppState {
    pool: Pool<AsyncMysqlConnection>
}

#[derive(Template)]
#[template(path="homepage.html")]
struct HomePageTemplate {
}


#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
    .with(
        tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            format!(
                "{}=debug,tower_http=debug,axum::rejection=trace",
                env!("CARGO_CRATE_NAME")
            )
            .into()
        }),
    )
    .with(tracing_subscriber::fmt::layer())
    .init();

    let app_state = AppState {
        pool: create_pool().await
    };
    

    let root_app = Router::new()
        .nest_service("/files", ServeDir::new("server_files").not_found_service(ServeFile::new("server_files\\static\\404.txt")))
        .route("/", get(index))
        .route("/sign-in", get(sign_in).post(process_sign_in))
        .route("/sign-up", get(sign_up).post(process_sign_up))
        .route("/browse", get(browse))
        .route("/browse/*product", get(product))
        .fallback_service(ServeFile::new("server_files\\static\\404.txt"))
        .layer(TraceLayer::new_for_http()).with_state(app_state);


    let listener = tokio::net::TcpListener::bind("127.0.0.1:1111").await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, root_app).await.unwrap();
}

async fn index() -> impl IntoResponse {
    let template = HomePageTemplate {};
    let html = template.render().unwrap();
    (StatusCode::OK, Html(html))
}



async fn create_pool() -> Pool<AsyncMysqlConnection> {
    dotenv().ok();
    let url = env::var("DATABASE_URL").expect("Environment variable DATABASE_URL must be set");
    let conf = AsyncDieselConnectionManager::<AsyncMysqlConnection>::new(&url);
    return Pool::builder(conf).build().unwrap_or_else(|_| panic!("Error creating pooled connection to db {}", url));
}


fn internal_error<E>(_err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, String::from("Interal Server Error"))
}
