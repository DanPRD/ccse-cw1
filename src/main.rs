#![allow(dead_code)]
use axum::{http::StatusCode, response::{Html, IntoResponse}, routing::get, Router};
use tower_http::{services::{ServeDir, ServeFile}, trace::TraceLayer};
use tracing;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use askama::Template;


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
    

    let root_app = Router::new()
        .nest_service("/favicon.ico", ServeFile::new("server_files\\favicon.ico"))
        .nest_service("/files", ServeDir::new("server_files").not_found_service(ServeFile::new("server_files\\static\\404.txt")))
        .route("/", get(index))
        .fallback_service(ServeFile::new("server_files\\static\\404.txt"))
        .layer(TraceLayer::new_for_http());


    let listener = tokio::net::TcpListener::bind("127.0.0.1:1111").await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, root_app).await.unwrap();
}

async fn index() -> impl IntoResponse {
    let template = HomePageTemplate {};
    let html = template.render().unwrap();
    (StatusCode::OK, Html(html))
}



