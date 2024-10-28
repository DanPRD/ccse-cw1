use askama::Template;
use axum::{extract::{Path, State}, http::StatusCode, response::{AppendHeaders, Html, IntoResponse}, Form};
use axum_extra::extract::CookieJar;
use bigdecimal::BigDecimal;
use diesel::{delete, dsl::insert_into, ExpressionMethods, QueryDsl};
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection, RunQueryDsl};
use serde::Deserialize;

use crate::{auth::session::validate_session, db::{models::{CartItem, Product}, schema::{cartitems, products}}, internal_error, logged_in, AppState, SESSION_COOKIE_NAME};

#[derive(Template)]
#[template(path="browse.html")]
struct BrowsePageTemplate {
    logged_in: bool,
    products: Vec<Product>
}

#[derive(Template)]
#[template(path="product.html")]
struct ProductPageTemplate {
    logged_in: bool,
    product: Product
}

#[derive(Template)]
#[template(path="cart.html")]
struct CartPageTemplate {
    logged_in: bool,
    products: Option<Vec<(Product, i32)>>,
    total_cost: Option<BigDecimal>
}

#[derive(Deserialize)]
pub enum Action {
    Add,
    Remove
}

#[derive(Deserialize)]
pub struct CartAction {
    product_id: i32,
    action: Action,
    quantity: i32
}




pub async fn browse(jar: CookieJar, State(state): State<AppState>) -> Result<(StatusCode, Html<String>), (StatusCode, String)> {
    let mut conn = state.pool.get().await.map_err(internal_error)?;
    let products: Vec<Product> = products::table.select(products::all_columns).limit(15).load(&mut conn).await.map_err(internal_error)?;
    let template = BrowsePageTemplate {products, logged_in: logged_in(&jar, &state.pool).await};
    let html = template.render().unwrap();
    Ok((StatusCode::OK, Html(html)))
}

pub async fn product(Path(path): Path<String>, jar: CookieJar,  State(state): State<AppState>) -> Result<(StatusCode, Html<String>), (StatusCode, String)> {
    let mut conn = state.pool.get().await.map_err(internal_error)?;
    let product: Product = products::table.select(products::all_columns).filter(products::imgname.eq(path + ".jpg")).first(&mut conn).await.map_err(|_| (StatusCode::NOT_FOUND, String::from("404 Not Found")))?;
    let template = ProductPageTemplate { product, logged_in: logged_in(&jar, &state.pool).await};
    let html = template.render().unwrap();
    Ok((StatusCode::OK, Html(html)))
}

pub async fn cart(jar: CookieJar, State(state): State<AppState>) -> Result<(StatusCode, Html<String>), (StatusCode, String)> {
    let template: CartPageTemplate;
    if let Some(token) = jar.get("sc-auth-session") {
        template = get_cart_items(&jar, token.value().to_owned(), &state.pool).await?;
    } else {
        template = CartPageTemplate {products: None, total_cost: None, logged_in: logged_in(&jar, &state.pool).await};
    }
    let html = template.render().unwrap();
    Ok((StatusCode::OK, Html(html)))    
}

async fn get_cart_items(jar: &CookieJar, session_token: String, pool: &Pool<AsyncMysqlConnection>) -> Result<CartPageTemplate, (StatusCode, String)> {
    let mut conn = pool.get().await.map_err(internal_error)?;
    let session = validate_session(session_token, pool).await?;
    let cartitems = products::table.inner_join(cartitems::table).select((products::all_columns, cartitems::quantity)).filter(cartitems::user_id.eq(session.user_id)).load::<(Product, i32)>(&mut conn).await.map_err(internal_error)?;
    if cartitems.len() == 0 {
        return Ok(CartPageTemplate {products: None, total_cost: None, logged_in: logged_in(&jar, &pool).await})
    }
    let total_cost: BigDecimal = cartitems.iter().map(|c| &c.0.cost * &c.1).sum();
    return Ok(CartPageTemplate {products: Some(cartitems), total_cost: Some(total_cost), logged_in: logged_in(&jar, &pool).await});
    
}

pub async fn cart_post_handler(jar: CookieJar, State(state): State<AppState>, Form(payload): Form<CartAction>) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut conn = state.pool.get().await.map_err(internal_error)?;
    let session_cookie= jar.get(SESSION_COOKIE_NAME).ok_or((StatusCode::UNAUTHORIZED, String::from("401 unauthorized")))?;
    let session = validate_session(session_cookie.value().to_owned(), &state.pool).await?;
    match payload.action {
        Action::Add => {
            if (1..=32).contains(&payload.quantity) {
                let entry = CartItem {product_id: payload.product_id, user_id: session.user_id, quantity: payload.quantity};
                insert_into(cartitems::table).values(entry).execute(&mut conn).await.map_err(internal_error)?;
                return Ok((StatusCode::OK, String::from("Added ✔")).into_response())

            } else {
                return Err((StatusCode::BAD_REQUEST, String::from("Invalid Quantity, please only add 1 to 32 of an item")))
            }
        }
        Action::Remove => {
            delete(cartitems::table).filter(cartitems::user_id.eq(session.user_id)).filter(cartitems::product_id.eq(payload.product_id)).execute(&mut conn).await.map_err(internal_error)?;
            return Ok(AppendHeaders([("HX-Location", "/cart")]).into_response())
        }
    }
}