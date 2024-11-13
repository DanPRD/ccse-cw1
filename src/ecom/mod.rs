use askama::Template;
use axum::{extract::{Path, State}, http::StatusCode, response::{AppendHeaders, Html, IntoResponse}};
use axum_extra::extract::{CookieJar, Form};
use bigdecimal::BigDecimal;
use diesel::{delete, dsl::insert_into, ExpressionMethods, QueryDsl};
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection, RunQueryDsl};
use serde::Deserialize;

use crate::{auth::session::validate_session, db::{models::{Address, CartProduct, Product}, schema::{cartproducts, likedproducts, products}}, internal_error, logged_in, AppState, SESSION_COOKIE_NAME};

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
    product: Product,
    is_liked: bool
}

#[derive(Template)]
#[template(path="cart.html")]
struct CartPageTemplate {
    logged_in: bool,
    products: Option<Vec<(Product, i32)>>,
    total_cost: Option<BigDecimal>
}

#[derive(Template)]
#[template(path="checkout.html")]
struct CheckoutPageTemplate {
    cartproducts: Option<Vec<(Product, i32)>>,
    logged_in: bool,
    saved_addresses: Option<Vec<Address>>,
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

#[derive(Deserialize)]
pub struct LikeAction {
    product_id: i32,
    action: Action
}

#[derive(Debug, Deserialize)]
pub struct CheckoutForm {
    cardnum: String, //check is valid num
    expiry: String,
    cvv: String, //check is valid num
    recipient_name: String,
    line_1: String,
    line_2: Option<String>,
    postcode: String,
    county: String,
    save_addr: Option<String>
}

impl CheckoutForm {
    fn verify_data(&mut self) -> Result<(), (StatusCode, String)> {
        self.cardnum.retain(|c| !c.is_whitespace());
        if !(self.cardnum.len() <= 16 && self.cardnum.parse::<u64>().is_ok()) { 
            return Err((StatusCode::BAD_REQUEST, String::from("Please enter a valid card number")))
        }
        self.cvv.retain(|c| !c.is_whitespace());
        if !(self.cvv.len() == 3 && self.cvv.parse::<u64>().is_ok()) {
            return Err((StatusCode::BAD_REQUEST, String::from("Please enter a valid cvv number")))
        }
        self.postcode.retain(|c| !c.is_whitespace());
        if !((self.postcode.len() <= 7 && self.postcode.len() >= 5)) {
            return Err((StatusCode::BAD_REQUEST, String::from("Please enter a valid postcode")))
        }
        let dates: Vec<&str> = self.expiry.split("/").collect();
        if dates.len() != 2 {
            return Err((StatusCode::BAD_REQUEST, String::from("Please enter a valid expiry date")))
        }
        let current_year = time::OffsetDateTime::now_utc().year()-2000;
        if !(dates[0].parse::<u8>().unwrap_or(255) <= 12 && (current_year as u8..=99).contains(&dates[1].parse::<u8>().unwrap_or(0))) {
            return Err((StatusCode::BAD_REQUEST, String::from("Please enter a valid expiry date")))
        }
        Ok(())
    }
}

/* 
#[derive(Debug)]
struct FormBool(bool);

impl<'de> Deserialize<'de> for FormBool {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de> {
        
        Deserialize::deserialize(deserializer).map(|val: &str| match val {
            "on" => FormBool(true),
            "off" => FormBool(false),
            _ => FormBool(false)
        })
    }
} */



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
    let is_liked;
    if let Some(session_cookie)= jar.get(SESSION_COOKIE_NAME) {
        let session = validate_session(session_cookie.value().to_owned(), &state.pool).await?;
        is_liked = likedproducts::table.select(likedproducts::all_columns).filter(likedproducts::product_id.eq(product.id)).filter(likedproducts::user_id.eq(session.user_id)).load::<(i32, i32)>(&mut conn).await.map_err(internal_error)?.len() > 0;
        println!("{}", is_liked)
    } else {
        is_liked = false
    }
    let template = ProductPageTemplate { product, logged_in: logged_in(&jar, &state.pool).await, is_liked};
    let html = template.render().unwrap();
    Ok((StatusCode::OK, Html(html)))
}

pub async fn cart(jar: CookieJar, State(state): State<AppState>) -> Result<(StatusCode, Html<String>), (StatusCode, String)> {
    let template: CartPageTemplate;
    if let Some(token) = jar.get(SESSION_COOKIE_NAME) {
        let (products, total_cost) = get_cart_items(token.value().to_owned(), &state.pool).await?;
        template = CartPageTemplate {products, total_cost, logged_in: logged_in(&jar, &state.pool).await};
    } else {
        template = CartPageTemplate {products: None, total_cost: None, logged_in: logged_in(&jar, &state.pool).await};
    }
    let html = template.render().unwrap();
    Ok((StatusCode::OK, Html(html)))    
}

async fn get_cart_items(session_token: String, pool: &Pool<AsyncMysqlConnection>) -> Result<(Option<Vec<(Product, i32)>>, Option<BigDecimal>), (StatusCode, String)> {
    let mut conn = pool.get().await.map_err(internal_error)?;
    let session = validate_session(session_token, pool).await?;
    let cartitems = products::table.inner_join(cartproducts::table).select((products::all_columns, cartproducts::quantity)).filter(cartproducts::user_id.eq(session.user_id)).load::<(Product, i32)>(&mut conn).await.map_err(internal_error)?;
    if cartitems.len() == 0 {
        return Ok((None, None))
    }
    let total_cost: BigDecimal = cartitems.iter().map(|c| &c.0.cost * &c.1).sum();
    return Ok((Some(cartitems), Some(total_cost)))
    
}

pub async fn cart_post_handler(jar: CookieJar, State(state): State<AppState>, Form(payload): Form<CartAction>) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut conn = state.pool.get().await.map_err(internal_error)?;
    let session_cookie= jar.get(SESSION_COOKIE_NAME).ok_or((StatusCode::UNAUTHORIZED, String::from("401 unauthorized")))?;
    let session = validate_session(session_cookie.value().to_owned(), &state.pool).await?;
    match payload.action {
        Action::Add => {
            if (1..=32).contains(&payload.quantity) {
                let entry = CartProduct {product_id: payload.product_id, user_id: session.user_id, quantity: payload.quantity};
                insert_into(cartproducts::table).values(entry).execute(&mut conn).await.map_err(internal_error)?;
                return Ok((StatusCode::OK, String::from("Added ✔")).into_response())

            } else {
                return Err((StatusCode::BAD_REQUEST, String::from("Invalid Quantity, please only add 1 to 32 of an item")))
            }
        }
        Action::Remove => {
            delete(cartproducts::table).filter(cartproducts::user_id.eq(session.user_id)).filter(cartproducts::product_id.eq(payload.product_id)).execute(&mut conn).await.map_err(internal_error)?;
            return Ok(AppendHeaders([("HX-Location", "/cart")]).into_response())
        }
    }
}

pub async fn liked(jar: CookieJar, State(state): State<AppState>) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut conn = state.pool.get().await.map_err(internal_error)?;
    let session_cookie= jar.get(SESSION_COOKIE_NAME).ok_or((StatusCode::UNAUTHORIZED, String::from("401 unauthorized")))?;
    let session = validate_session(session_cookie.value().to_owned(), &state.pool).await?;
    let products = products::table.select(products::all_columns).inner_join(likedproducts::table).filter(likedproducts::user_id.eq(session.user_id)).load::<Product>(&mut conn).await.map_err(internal_error)?;
    let template = BrowsePageTemplate {products, logged_in: logged_in(&jar, &state.pool).await};
    let html = template.render().unwrap();
    Ok((StatusCode::OK, Html(html)))
}

pub async fn like_post_handler(jar: CookieJar, State(state): State<AppState>, Form(payload): Form<LikeAction>) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut conn = state.pool.get().await.map_err(internal_error)?;
    let session_cookie= jar.get(SESSION_COOKIE_NAME).ok_or((StatusCode::UNAUTHORIZED, String::from("401 unauthorized")))?;
    let session = validate_session(session_cookie.value().to_owned(), &state.pool).await?;
    match payload.action {

        Action::Add => {
            insert_into(likedproducts::table).values((likedproducts::product_id.eq(payload.product_id), likedproducts::user_id.eq(session.user_id))).execute(&mut conn).await.map_err(internal_error)?;
            return Ok("Added ✔")
        }

        Action::Remove => {
            delete(likedproducts::table).filter(likedproducts::user_id.eq(session.user_id)).filter(likedproducts::product_id.eq(payload.product_id)).execute(&mut conn).await.map_err(internal_error)?;
            return Ok("Removed ✔")
        }

    }
}

pub async fn checkout(jar: CookieJar, State(state): State<AppState>) -> Result<impl IntoResponse, (StatusCode, String)>  {
    let session_cookie= jar.get(SESSION_COOKIE_NAME).ok_or((StatusCode::UNAUTHORIZED, String::from("401 unauthorized")))?;
    let (cartproducts, total_cost) = get_cart_items(session_cookie.value().to_owned(), &state.pool).await?;
    let template = CheckoutPageTemplate {logged_in: logged_in(&jar, &state.pool).await, saved_addresses: None, cartproducts, total_cost};
    let html = template.render().unwrap();
    Ok((StatusCode::OK, Html(html)))
}

pub async fn checkout_post_handler(jar: CookieJar, State(state): State<AppState>, Form(mut payload): Form<CheckoutForm>) -> Result<String, (StatusCode, String)> {
    let session_cookie= jar.get(SESSION_COOKIE_NAME).ok_or((StatusCode::UNAUTHORIZED, String::from("401 unauthorized")))?;
    let session = validate_session(session_cookie.value().to_owned(), &state.pool).await?;
    payload.verify_data()?;
    return Ok(String::from("Bought Item!"))
}   