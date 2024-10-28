#![allow(dead_code)]
use axum::{extract::State, http::StatusCode, response::{Html, IntoResponse}, routing::{get, post}, Router};
use axum_extra::extract::CookieJar;
use ecom::{browse, cart, cart_post_handler, product};
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
use auth::{session::validate_session, signin::{process_sign_in, sign_in}, signout::sign_out, signup::{process_sign_up, sign_up}};

const SESSION_COOKIE_NAME: &str = "sc-auth-session";

#[derive(Clone)]
struct AppState {
    pool: Pool<AsyncMysqlConnection>
}

#[derive(Template)]
#[template(path="homepage.html")]
struct HomePageTemplate {
    logged_in: bool
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
        .route("/sign-out", post(sign_out))
        .route("/browse", get(browse))
        .route("/cart", get(cart).post(cart_post_handler))
        .route("/browse/:product", get(product))
        .fallback_service(ServeFile::new("server_files\\static\\404.txt"))
        .layer(TraceLayer::new_for_http()).with_state(app_state);


    let listener = tokio::net::TcpListener::bind("127.0.0.1:1111").await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, root_app).await.unwrap();
}

async fn index(jar: CookieJar, State(state): State<AppState>) -> impl IntoResponse {
    let template = HomePageTemplate {logged_in: logged_in(&jar, &state.pool).await};
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

async fn logged_in(jar: &CookieJar, pool: &Pool<AsyncMysqlConnection>) -> bool {
    if let Some(cookie) = jar.get(SESSION_COOKIE_NAME) {
        return validate_session(cookie.value().to_owned(), pool).await.is_ok()
    }
    false
}
