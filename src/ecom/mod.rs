use askama::Template;
use axum::{extract::{Path, State}, http::StatusCode, response::Html};
use axum_extra::extract::CookieJar;
use bigdecimal::BigDecimal;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection, RunQueryDsl};

use crate::{auth::session::validate_session, db::{models::Product, schema::{cartitems, products}}, internal_error, AppState};

#[derive(Template)]
#[template(path="browse.html")]
struct BrowsePageTemplate {
    products: Vec<Product>
}

#[derive(Template)]
#[template(path="product.html")]
struct ProductPageTemplate {
    product: Product
}

#[derive(Template)]
#[template(path="cart.html")]
struct CartPageTemplate {
    products: Option<Vec<(Product, i32)>>,
    total_cost: Option<BigDecimal>
}



pub async fn browse(State(state): State<AppState>) -> Result<(StatusCode, Html<String>), (StatusCode, String)> {
    let mut conn = state.pool.get().await.map_err(internal_error)?;
    let products: Vec<Product> = products::table.select(products::all_columns).limit(15).load(&mut conn).await.map_err(internal_error)?;
    let template = BrowsePageTemplate {products};
    let html = template.render().unwrap();
    Ok((StatusCode::OK, Html(html)))
}

pub async fn product(Path(path): Path<String>, State(state): State<AppState>) -> Result<(StatusCode, Html<String>), (StatusCode, String)> {
    let mut conn = state.pool.get().await.map_err(internal_error)?;
    let product: Product = products::table.select(products::all_columns).filter(products::imgname.eq(path + ".jpg")).first(&mut conn).await.map_err(|_| (StatusCode::NOT_FOUND, String::from("404 Not Found")))?;
    let template = ProductPageTemplate { product};
    let html = template.render().unwrap();
    Ok((StatusCode::OK, Html(html)))
}

pub async fn cart(jar: CookieJar, State(state): State<AppState>) -> Result<(StatusCode, Html<String>), (StatusCode, String)> {
    let template: CartPageTemplate;
    if let Some(token) = jar.get("sc-auth-session") {
        template = get_cart_items(token.value().to_owned(), &state.pool).await?;
    } else {
        template = CartPageTemplate {products: None, total_cost: None};
    }
    let html = template.render().unwrap();
    Ok((StatusCode::OK, Html(html)))
}

async fn get_cart_items(session_token: String, pool: &Pool<AsyncMysqlConnection>) -> Result<CartPageTemplate, (StatusCode, String)> {
    let mut conn = pool.get().await.map_err(internal_error)?;
    let session = validate_session(session_token, pool).await?;
    let cartitems = products::table.inner_join(cartitems::table).select((products::all_columns, cartitems::quantity)).filter(cartitems::user_id.eq(session.user_id)).load::<(Product, i32)>(&mut conn).await.map_err(internal_error)?;
    if cartitems.len() == 0 {
        return Ok(CartPageTemplate {products: None, total_cost: None})
    }
    let total_cost: BigDecimal = cartitems.iter().map(|c| &c.0.cost * &c.1).sum();
    return Ok(CartPageTemplate {products: Some(cartitems), total_cost: Some(total_cost)});
    
}