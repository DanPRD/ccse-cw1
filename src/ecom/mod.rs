use askama::Template;
use axum::{extract::{Path, State}, http::StatusCode, response::{Html, IntoResponse}};
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;

use crate::{db::{models::{ListedProduct, Product}, schema::products}, internal_error, AppState};

#[derive(Template)]
#[template(path="browse.html")]
struct BrowsePageTemplate {
    products: Vec<ListedProduct>
}

#[derive(Template)]
#[template(path="product.html")]
struct ProductPageTemplate {
    product: Product
}


pub async fn browse(State(state): State<AppState>) -> Result<(StatusCode, Html<String>), (StatusCode, String)> {
    let mut conn = state.pool.get().await.map_err(internal_error)?;
    let products: Vec<ListedProduct> = products::table.select((products::title, products::imgname)).limit(15).load(&mut conn).await.map_err(internal_error)?;
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