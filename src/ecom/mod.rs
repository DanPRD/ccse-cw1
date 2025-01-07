use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{AppendHeaders, Html, IntoResponse},
};
use axum_extra::extract::{CookieJar, Form};
use bigdecimal::BigDecimal;
use diesel::{delete, dsl::exists, insert_into, sql_query, ExpressionMethods, QueryDsl};
use diesel_async::{pooled_connection::deadpool::Pool, AsyncPgConnection, RunQueryDsl};
use serde::Deserialize;

use crate::{
    auth::session::validate_session,
    db::{
        models::{Address, CartProduct, Order, OrderWithId, Product},
        schema::{addresses, cartproducts, likedproducts, orders, productorders, products},
    },
    internal_error, logged_in, AppState, SESSION_COOKIE_NAME,
};
pub mod admin;

#[derive(Template)]
#[template(path = "browse.html")]
struct BrowsePageTemplate {
    logged_in: bool,
    products: Vec<Product>,
}

#[derive(Template)]
#[template(path = "product.html")]
struct ProductPageTemplate {
    logged_in: bool,
    product: Product,
    is_liked: bool,
}

#[derive(Template)]
#[template(path = "cart.html")]
struct CartPageTemplate {
    logged_in: bool,
    products: Option<Vec<(Product, i32)>>,
    total_cost: Option<BigDecimal>,
}

#[derive(Template)]
#[template(path = "checkout.html")]
struct CheckoutPageTemplate {
    cartproducts: Option<Vec<(Product, i32)>>,
    logged_in: bool,
    saved_addresses: Option<Vec<Address>>,
    total_cost: Option<BigDecimal>,
}

#[derive(Template)]
#[template(path = "orders.html")]
struct OrderPageTemplate {
    logged_in: bool,
    orders: Option<Vec<OrderInfo>>,
}

#[derive(Template)]
#[template(path = "success_checkout.html")]
struct CheckoutSuccess;

#[derive(Template)]
#[template(path = "order_details.html")]
struct OrderDetails {
    order_info: OrderInfo,
}

#[derive(Deserialize)]
pub enum Action {
    Add,
    Remove,
}

#[derive(Deserialize)]
pub struct CartAction {
    product_id: i32,
    action: Action,
    quantity: i32,
}

#[derive(Deserialize)]
pub struct LikeAction {
    product_id: i32,
    action: Action,
}

pub struct OrderInfo {
    info: OrderWithId,
    address: Address,
    products: Vec<(Product, i32)>,
    total: BigDecimal,
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
    save_addr: Option<String>,
}

#[derive(Deserialize)]
pub struct OrderDetailsForm {
    order_id: i32,
}

impl CheckoutForm {
    fn verify_data(&mut self) -> Result<(), (StatusCode, String)> {
        self.cardnum.retain(|c| !c.is_whitespace());
        if !(self.cardnum.len() <= 16 && self.cardnum.parse::<u64>().is_ok()) {
            return Err((
                StatusCode::BAD_REQUEST,
                String::from("Please enter a valid card number"),
            ));
        }
        self.cvv.retain(|c| !c.is_whitespace());
        if !(self.cvv.len() == 3 && self.cvv.parse::<u64>().is_ok()) {
            return Err((
                StatusCode::BAD_REQUEST,
                String::from("Please enter a valid cvv number"),
            ));
        }
        self.postcode.retain(|c| !c.is_whitespace());
        if !(self.postcode.len() <= 7 && self.postcode.len() >= 5) {
            return Err((
                StatusCode::BAD_REQUEST,
                String::from("Please enter a valid postcode"),
            ));
        }
        let dates: Vec<&str> = self.expiry.split("/").collect();
        if dates.len() != 2 {
            return Err((
                StatusCode::BAD_REQUEST,
                String::from("Please enter a valid expiry date"),
            ));
        }
        let current_year = time::OffsetDateTime::now_utc().year() - 2000;
        if !(dates[0].parse::<u8>().unwrap_or(255) <= 12
            && (current_year as u8..=99).contains(&dates[1].parse::<u8>().unwrap_or(0)))
        {
            return Err((
                StatusCode::BAD_REQUEST,
                String::from("Please enter a valid expiry date"),
            ));
        }
        if self.recipient_name.is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                String::from("Please enter your name"),
            ));
        }
        if self.line_1.is_empty() || self.county.is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                String::from("Please enter your address"),
            ));
        }
        Ok(())
    }

    fn parse_address(self, user_id: i32) -> Address {
        Address {
            user_id,
            recipient_name: self.recipient_name,
            line_1: self.line_1,
            line_2: self.line_2.unwrap_or_default(),
            postcode: self.postcode,
            county: self.county,
        }
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

pub async fn browse(
    jar: CookieJar,
    State(state): State<AppState>,
) -> Result<(StatusCode, Html<String>), (StatusCode, String)> {
    let mut conn = state.pool.get().await.map_err(internal_error)?;
    let products: Vec<Product> = products::table
        .select(products::all_columns)
        .filter(products::listed.eq(true))
        .load(&mut conn)
        .await
        .map_err(internal_error)?;
    let template = BrowsePageTemplate {
        products,
        logged_in: logged_in(&jar, &state.pool).await,
    };
    let html = template.render().unwrap();
    Ok((StatusCode::OK, Html(html)))
}

pub async fn product(
    Path(path): Path<String>,
    jar: CookieJar,
    State(state): State<AppState>,
) -> Result<(StatusCode, Html<String>), (StatusCode, String)> {
    let mut conn = state.pool.get().await.map_err(internal_error)?;
    let product: Product = products::table
        .select(products::all_columns)
        .filter(products::imgname.eq(path + ".jpg"))
        .first(&mut conn)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, String::from("404 Not Found")))?;
    let is_liked;
    if let Some(session_cookie) = jar.get(SESSION_COOKIE_NAME) {
        let session = validate_session(session_cookie.value().to_owned(), &state.pool).await?;
        is_liked = likedproducts::table
            .select(likedproducts::all_columns)
            .filter(likedproducts::product_id.eq(product.id))
            .filter(likedproducts::user_id.eq(session.user_id))
            .load::<(i32, i32)>(&mut conn)
            .await
            .map_err(internal_error)?
            .len()
            > 0;
    } else {
        is_liked = false
    }
    let template = ProductPageTemplate {
        product,
        logged_in: logged_in(&jar, &state.pool).await,
        is_liked,
    };
    let html = template.render().unwrap();
    Ok((StatusCode::OK, Html(html)))
}

pub async fn cart(
    jar: CookieJar,
    State(state): State<AppState>,
) -> Result<(StatusCode, Html<String>), (StatusCode, String)> {
    let session_cookie = jar
        .get(SESSION_COOKIE_NAME)
        .ok_or((StatusCode::UNAUTHORIZED, String::from("401 unauthorized")))?;
    let (products, total_cost) = get_cart_items(session_cookie.value().to_owned(), &state.pool).await?;
    let template = CartPageTemplate {
        products,
        total_cost,
        logged_in: true,
    };
    let html = template.render().unwrap();
    Ok((StatusCode::OK, Html(html)))
}

async fn get_cart_items(
    session_token: String,
    pool: &Pool<AsyncPgConnection>,
) -> Result<(Option<Vec<(Product, i32)>>, Option<BigDecimal>), (StatusCode, String)> {
    let mut conn = pool.get().await.map_err(internal_error)?;
    let session = validate_session(session_token, pool).await?;
    let cartitems = products::table
        .inner_join(cartproducts::table)
        .select((products::all_columns, cartproducts::quantity))
        .filter(cartproducts::user_id.eq(session.user_id))
        .load::<(Product, i32)>(&mut conn)
        .await
        .map_err(internal_error)?;
    if cartitems.len() == 0 {
        return Ok((None, None));
    }
    let total_cost: BigDecimal = cartitems.iter().map(|c| &c.0.cost * &c.1).sum();
    return Ok((Some(cartitems), Some(total_cost)));
}

pub async fn cart_post_handler(
    jar: CookieJar,
    State(state): State<AppState>,
    Form(payload): Form<CartAction>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut conn = state.pool.get().await.map_err(internal_error)?;
    let session_cookie = jar
        .get(SESSION_COOKIE_NAME)
        .ok_or((StatusCode::UNAUTHORIZED, String::from("401 unauthorized")))?;
    let session = validate_session(session_cookie.value().to_owned(), &state.pool).await?;
    match payload.action {
        Action::Add => {
            if (1..=32).contains(&payload.quantity) {
                let sub_query = cartproducts::table
                    .select(cartproducts::product_id)
                    .filter(cartproducts::product_id.eq(payload.product_id))
                    .filter(cartproducts::user_id.eq(session.user_id));
                let product: (bool, bool) = products::table
                    .select((products::listed, exists(sub_query)))
                    .filter(products::id.eq(payload.product_id))
                    .first(&mut conn).await
                    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, String::from("500 Internal Server Error")))?;
                if product.1 { // is it in the users cart?
                    return Err((StatusCode::BAD_REQUEST,
                        String::from("This item is already in your cart"),
                    ));
                }
                if !product.0 { // is it listed?
                    return Err((StatusCode::BAD_REQUEST,
                        String::from("This item is no longer listed"),
                    ));
                }
                let entry = CartProduct {product_id: payload.product_id,user_id: session.user_id,
                    quantity: payload.quantity,
                };
                insert_into(cartproducts::table)
                    .values(entry)
                    .on_conflict_do_nothing()
                    .execute(&mut conn).await
                    .map_err(internal_error)?;
                return Ok((StatusCode::OK, String::from("Added ✔")).into_response());
            } else {
                return Err((
                    StatusCode::BAD_REQUEST,
                    String::from("Invalid Quantity, please only add 1 to 32 of an item"),
                ));
            }
        }
        Action::Remove => {
            delete(cartproducts::table)
                .filter(cartproducts::user_id.eq(session.user_id))
                .filter(cartproducts::product_id.eq(payload.product_id))
                .execute(&mut conn).await
                .map_err(internal_error)?;
            return Ok(AppendHeaders([("HX-Location", "/cart")]).into_response());
        }
    }
}

pub async fn liked(
    jar: CookieJar,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut conn = state.pool.get().await.map_err(internal_error)?;
    let session_cookie = jar
        .get(SESSION_COOKIE_NAME)
        .ok_or((StatusCode::UNAUTHORIZED, String::from("401 unauthorized")))?;
    let session = validate_session(session_cookie.value().to_owned(), &state.pool).await?;
    let products = products::table
        .select(products::all_columns)
        .inner_join(likedproducts::table)
        .filter(likedproducts::user_id.eq(session.user_id))
        .load::<Product>(&mut conn)
        .await
        .map_err(internal_error)?;
    let template = BrowsePageTemplate {
        products,
        logged_in: true,
    };
    let html = template.render().unwrap();
    Ok((StatusCode::OK, Html(html)))
}

pub async fn like_post_handler(
    jar: CookieJar,
    State(state): State<AppState>,
    Form(payload): Form<LikeAction>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut conn = state.pool.get().await.map_err(internal_error)?;
    let session_cookie = jar
        .get(SESSION_COOKIE_NAME)
        .ok_or((StatusCode::UNAUTHORIZED, String::from("401 unauthorized")))?;
    let session = validate_session(session_cookie.value().to_owned(), &state.pool).await?;
    match payload.action {
        Action::Add => {
            insert_into(likedproducts::table)
                .values((
                    likedproducts::product_id.eq(payload.product_id),
                    likedproducts::user_id.eq(session.user_id),
                ))
                .execute(&mut conn)
                .await
                .map_err(internal_error)?;
            return Ok("Added ✔");
        }

        Action::Remove => {
            delete(likedproducts::table)
                .filter(likedproducts::user_id.eq(session.user_id))
                .filter(likedproducts::product_id.eq(payload.product_id))
                .execute(&mut conn)
                .await
                .map_err(internal_error)?;
            return Ok("Removed ✔");
        }
    }
}

pub async fn checkout(
    jar: CookieJar,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let session_cookie = jar
        .get(SESSION_COOKIE_NAME)
        .ok_or((StatusCode::UNAUTHORIZED, String::from("401 unauthorized")))?;
    let (cartproducts, total_cost) =
        get_cart_items(session_cookie.value().to_owned(), &state.pool).await?;
    let template = CheckoutPageTemplate {
        logged_in: logged_in(&jar, &state.pool).await,
        saved_addresses: None,
        cartproducts,
        total_cost,
    };
    let html = template.render().unwrap();
    Ok((StatusCode::OK, Html(html)))
}

pub async fn checkout_post_handler(
    jar: CookieJar,
    State(state): State<AppState>,
    Form(mut payload): Form<CheckoutForm>,
) -> Result<String, (StatusCode, String)> {
    let mut conn = state.pool.get().await.map_err(internal_error)?;
    let session_cookie = jar
        .get(SESSION_COOKIE_NAME)
        .ok_or((StatusCode::UNAUTHORIZED, String::from("401 unauthorized")))?;
    let session = validate_session(session_cookie.value().to_owned(), &state.pool).await?;
    payload.verify_data()?;
    if cartproducts::table
        .filter(cartproducts::user_id.eq(session.user_id))
        .count()
        .get_result::<i64>(&mut conn).await
        .map_err(internal_error)?
        == 0
    {
        return Err((
            StatusCode::BAD_REQUEST,
            String::from("Please add items to your cart to buy them!"),
        ));
    }
    let address_id = insert_into(addresses::table)
        .values(payload.parse_address(session.user_id))
        .returning(addresses::id)
        .get_result::<i32>(&mut conn).await
        .map_err(internal_error)?;
    let order_id = insert_into(orders::table)
        .values(Order {
            user_id: session.user_id,
            address_id,
        })
        .returning(orders::id)
        .get_result::<i32>(&mut conn).await
        .map_err(internal_error)?;
    sql_query(format!("insert into productorders select product_id, {}, quantity from cartproducts where cartproducts.user_id = {};",
        order_id as i32,
        session.user_id))
        .execute(&mut conn).await.map_err(internal_error)?;
    diesel::delete(cartproducts::table)
        .filter(cartproducts::user_id.eq(session.user_id))
        .execute(&mut conn)
        .await
        .map_err(internal_error)?;
    let html = CheckoutSuccess.render().unwrap();
    return Ok(html);
}

pub async fn orders(
    jar: CookieJar,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut usr_orders = vec![];
    let mut conn = state.pool.get().await.map_err(internal_error)?;
    let session_cookie = jar
        .get(SESSION_COOKIE_NAME)
        .ok_or((StatusCode::UNAUTHORIZED, String::from("401 unauthorized")))?;
    let session = validate_session(session_cookie.value().to_owned(), &state.pool).await?;
    let orders = orders::table
        .select(orders::all_columns)
        .filter(orders::user_id.eq(session.user_id))
        .load::<OrderWithId>(&mut conn).await
        .map_err(internal_error)?;
    for order in orders {
        let addr = addresses::table
            .select((
                addresses::user_id,
                addresses::recipient_name,
                addresses::line_1,
                addresses::line_2,
                addresses::postcode,
                addresses::county,
            ))
            .filter(addresses::id.eq(order.address_id))
            .first::<Address>(&mut conn).await
            .map_err(internal_error)?;
        usr_orders.push(OrderInfo {
            info: order,
            address: addr,
            products: vec![],
            total: 0.into(),
        })
    }
    let usr_orders = if usr_orders.is_empty() {
        None
    } else {
        Some(usr_orders)
    };
    let template = OrderPageTemplate {
        logged_in: true,
        orders: usr_orders,
    };
    let html = template.render().unwrap();
    Ok(Html(html))
}

pub async fn view_order_details(
    jar: CookieJar,
    State(state): State<AppState>,
    Form(payload): Form<OrderDetailsForm>,
) -> Result<Html<String>, (StatusCode, String)> {
    let mut conn = state.pool.get().await.map_err(internal_error)?;
    let session_cookie = jar
        .get(SESSION_COOKIE_NAME)
        .ok_or((StatusCode::UNAUTHORIZED, String::from("401 unauthorized")))?;
    let session = validate_session(session_cookie.value().to_owned(), &state.pool).await?;
    let address: Address = addresses::table
        .select((
            addresses::user_id,
            addresses::recipient_name,
            addresses::line_1,
            addresses::line_2,
            addresses::postcode,
            addresses::county,
        ))
        .inner_join(orders::table)
        .filter(orders::id.eq(payload.order_id))
        .filter(orders::user_id.eq(session.user_id))
        .first::<Address>(&mut conn)
        .await
        .map_err(internal_error)?;
    let products = productorders::table
        .inner_join(products::table)
        .select((products::all_columns, productorders::quantity))
        .filter(productorders::order_id.eq(payload.order_id))
        .load::<(Product, i32)>(&mut conn)
        .await
        .map_err(internal_error)?;
    let total: BigDecimal = products.iter().map(|c| &c.0.cost * &c.1).sum();
    let html = OrderDetails {
        order_info: OrderInfo {
            address,
            info: OrderWithId::default(),
            products,
            total,
        },
    }
    .render()
    .unwrap();
    Ok(Html(html))
}
