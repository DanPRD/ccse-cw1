use std::{default, str::FromStr};

use askama::Template;
use axum::{body::Bytes, extract::{multipart::MultipartError, Multipart, Path, State }, http::StatusCode, response::{AppendHeaders, Html, IntoResponse, Response}, routing::{get, post, MethodRouter}, Router};
use axum_extra::extract::{CookieJar, Form};
use bigdecimal::BigDecimal;
use diesel::{debug_query, delete, expression::AsExpression, insert_into, sql_query, sql_types::Integer, update, ExpressionMethods, Insertable, IntoSql, QueryDsl};
use diesel_async::{pooled_connection::deadpool::Pool, AsyncPgConnection, RunQueryDsl};
use serde::Deserialize;
use sha2::Digest;
use tokio::{fs, io::AsyncWriteExt};

use crate::{auth::session::validate_session, db::{models::{Address, CartProduct, NewProduct, Order, OrderWithId, Product, Session}, schema::{addresses, cartproducts, likedproducts, orders, productorders, products, sessions, users}}, internal_error, logged_in, AppState, SESSION_COOKIE_NAME};

#[derive(Template)]
#[template(path="admin.html")]
struct AdminDashboardPage {
    products: Vec<Product>
}

#[derive(Default)]
struct AddProductForm {
    image: Bytes,
    image_title: String,
    title: String,
    description: String,
    cost: BigDecimal
}

#[derive(Deserialize)]
struct ProductForm {
    id: i32
}

impl AddProductForm {
    async fn parse_from_multipart(mut form: Multipart) -> Result<Self, Response> {
        let mut ret = Self::default();
        while let Some(field) = form.next_field().await.unwrap() {
            match field.name() {
                Some(name) => {
                    match name {
                        "image" => {
                            ret.image_title = field.file_name().ok_or((StatusCode::BAD_REQUEST, String::from("Incorrect Fields")).into_response())?.to_owned();
                            ret.image = field.bytes().await.map_err(|e| e.into_response())?;
                        }
                        "title" => {
                            ret.title = field.text().await.map_err(|e| e.into_response())?
                        }
                        "description" => {
                            ret.description = field.text().await.map_err(|e| e.into_response())?
                        }
                        "cost" => {
                            ret.cost = BigDecimal::from_str(&field.text().await.map_err(|e| e.into_response())?)
                                .map_err(|_| (StatusCode::BAD_REQUEST, String::from("Incorrect Fields")).into_response())?;
                        }
                        _ => ()
                    }
                }
                None => ()
            }
        }

        if ret.image.len() == 0 || ret.title.len() == 0 || ret.description.len() == 0 {
            return Err((StatusCode::BAD_REQUEST, String::from("Incorrect Fields")).into_response())
        }

        return Ok(ret)
    }

    fn to_sql_insert(self) -> NewProduct {
        NewProduct {
            id: None,
            title: self.title,
            description: self.description,
            imgname: self.image_title,
            cost: self.cost
        }
    }
}

pub fn admin_routes() -> Router<AppState> {
    Router::new()
    .route("/", get(admin_dashboard))
    .route("/addproduct", post(handle_add_product))
    .route("/removeproduct", post(handle_remove_product))
    .route("/unlist", post(handle_unlist_product))
    .route("/relist", post(handle_relist_product))
}


async fn validate_admin(jar: CookieJar, pool: &Pool<AsyncPgConnection>) -> Result<Session, (StatusCode, String)> {
    if let Some(cookie) = jar.get(SESSION_COOKIE_NAME) {
        let token = cookie.value();
        let mut conn = pool.get().await.map_err(internal_error)?;
        let mut hasher = sha2::Sha256::new();
        hasher.update(token.as_bytes());
        let session_id = hex::encode(hasher.finalize());
        // "SELECT user_session.id, user_session.user_id, user_session.expires_at, user.id FROM user_session INNER JOIN user ON user.id = user_session.user_id WHERE id = ?"
        let session: (Session, bool) = sessions::table.inner_join(users::table).select((sessions::all_columns, users::is_admin)).filter(sessions::id.eq(&session_id)).first(&mut conn).await.map_err(|_| (StatusCode::UNAUTHORIZED, String::from("401 Unauthorized")))?;

        if time::OffsetDateTime::now_utc() > session.0.expires_at {
            delete(sessions::table).filter(sessions::id.eq(session_id)).execute(&mut conn).await.map_err(internal_error)?;
        }
        if session.1 == true {
            return Ok(session.0)
        }
    }
    return Err((StatusCode::BAD_REQUEST, String::from("401 Unauthorized")))
}


async fn admin_dashboard(jar: CookieJar, State(state): State<AppState>) -> Result<(StatusCode, Html<String>), (StatusCode, String)> {
    let session = validate_admin(jar, &state.pool).await?;
    let mut conn = state.pool.get().await.map_err(internal_error)?;
    let products = products::table.select(products::all_columns).order(products::id.asc()).load(&mut conn).await.map_err(internal_error)?;
    let template = AdminDashboardPage {products};
    let html = template.render().unwrap();
    Ok((StatusCode::OK, Html(html)))

}

async fn handle_add_product(jar: CookieJar, State(state): State<AppState>, mut form: Multipart) -> Result<Response, (StatusCode, String)> {
    let session = validate_admin(jar, &state.pool).await?;
    let mut conn = state.pool.get().await.map_err(internal_error)?;
    let form = AddProductForm::parse_from_multipart(form).await.map_err( |_| (StatusCode::BAD_REQUEST, String::from("Incorrect form fields")))?;
    let mut file = fs::File::options().write(true).create_new(true).open(["server_files\\images\\", &form.image_title].concat()).await.map_err(internal_error)?;
    file.write_all(&form.image).await.map_err(internal_error)?;
    insert_into(products::table).values(form.to_sql_insert()).execute(&mut conn).await.map_err(internal_error)?;
    Ok(AppendHeaders([("HX-Location", "/adminpanel")]).into_response())

}

async fn handle_remove_product(jar: CookieJar, State(state): State<AppState>, Form(form): Form<ProductForm>) -> Result<Response, (StatusCode, String)> {
    let session = validate_admin(jar, &state.pool).await?;
    let mut conn = state.pool.get().await.map_err(internal_error)?;
    let img: String = delete(products::table).filter(products::id.eq(form.id)).returning(products::imgname).get_result(&mut conn).await.map_err(|_| (StatusCode::CONFLICT, String::from("Unable to remove product from database due to foreign key constraints")))?;
    fs::remove_file(["server_files\\images\\", &img].concat()).await.map_err(internal_error)?;
    Ok(AppendHeaders([("HX-Location", "/adminpanel")]).into_response())
    
}

async fn handle_unlist_product(jar: CookieJar, State(state): State<AppState>, Form(form): Form<ProductForm>) -> Result<Response, (StatusCode, String)> {
    let session = validate_admin(jar, &state.pool).await?;
    let mut conn = state.pool.get().await.map_err(internal_error)?;
    update(products::table).set(products::listed.eq(false)).filter(products::id.eq(form.id)).execute(&mut conn).await.map_err(internal_error)?;
    Ok(AppendHeaders([("HX-Location", "/adminpanel")]).into_response())
}

async fn handle_relist_product(jar: CookieJar, State(state): State<AppState>, Form(form): Form<ProductForm>) -> Result<Response, (StatusCode, String)> {
    let session = validate_admin(jar, &state.pool).await?;
    let mut conn = state.pool.get().await.map_err(internal_error)?;
    update(products::table).set(products::listed.eq(true)).filter(products::id.eq(form.id)).execute(&mut conn).await.map_err(internal_error)?;

    Ok(AppendHeaders([("HX-Location", "/adminpanel")]).into_response())
}