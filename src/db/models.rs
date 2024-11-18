use bigdecimal::BigDecimal;
use diesel::prelude::*;
use crate::db::schema::{addresses, cartproducts, orders, productorders, products, sessions, users};

#[derive(Queryable, Selectable)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: i32,
    pub email: String,
    pub password: String,
    pub is_admin: bool
}

#[derive(Insertable)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewUser<'a> {
    pub email: &'a str,
    pub password: &'a str,
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Session {
    pub id: String,
    pub user_id: i32,
    pub expires_at: time::OffsetDateTime
}

#[derive(Queryable, Selectable, Insertable, Clone, Debug)]
#[diesel(table_name = products)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Product {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub imgname: String,
    pub cost: BigDecimal
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = addresses)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Address {
    pub user_id: i32,
    pub recipient_name: String,
    pub line_1: String,
    pub line_2: String,
    pub postcode: String,
    pub county: String
}

#[derive(Identifiable, Selectable, Queryable, Associations, Insertable)]
#[diesel(belongs_to(User))]
#[diesel(belongs_to(Product))]
#[diesel(table_name = cartproducts)]
#[diesel(primary_key(user_id, product_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CartProduct {
    pub user_id: i32,
    pub product_id: i32,
    pub quantity: i32
}

#[derive(Insertable)]
#[diesel(table_name = orders)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Order {
    pub user_id: i32,
    pub address_id: i32
}

#[derive(Queryable)]
#[diesel(table_name = orders)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct OrderWithId {
    pub id: i32,
    pub user_id: i32,
    pub address_id: i32
}

#[derive(Insertable, Selectable, Queryable)]
#[diesel(table_name = productorders)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ProductInOrder {
    pub product_id: i32,
    pub order_id: i32,
    pub quantity: i32
}

