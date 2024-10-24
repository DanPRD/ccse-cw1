use bigdecimal::BigDecimal;
use diesel::prelude::*;
use crate::db::schema::{products, sessions, users};

#[derive(Queryable, Selectable)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct User {
    pub id: i32,
    pub email: String,
    pub password: String,
    pub is_admin: bool
}

#[derive(Insertable)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct NewUser<'a> {
    pub email: &'a str,
    pub password: &'a str,
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = sessions)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Session {
    pub id: String,
    pub user_id: i32,
    pub expires_at: time::OffsetDateTime
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = products)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Product {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub imgname: String,
    pub cost: BigDecimal
}
