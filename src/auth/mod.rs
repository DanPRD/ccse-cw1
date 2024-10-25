pub mod session;
pub mod signin {

    use argon2::{Argon2, PasswordHash, PasswordVerifier};
    use askama::Template;
    use axum::{extract::State, http::StatusCode, response::{AppendHeaders, Html, IntoResponse}, Form};
    use axum_extra::extract::{cookie::{Cookie, SameSite}, CookieJar};
    use diesel::{ExpressionMethods, QueryDsl};
    use diesel_async::RunQueryDsl;
    use serde::Deserialize;
    use ::time::Duration;
    use crate::{auth::session::create_session, db::schema::users, internal_error, AppState};
    #[derive(Template)]
    #[template(path="signin.html")]
    struct SignInPage {

    }

    #[derive(Debug, Deserialize)]
    pub struct SignInData {
        email: String,
        password: String
    }

    pub async fn sign_in() -> impl IntoResponse {
        let template = SignInPage {};
        let html = template.render().unwrap();
        return (StatusCode::OK, Html(html))
    }

    pub async fn process_sign_in(state: State<AppState>, jar: CookieJar, Form(sign_in_form): Form<SignInData>) -> Result<impl IntoResponse, (StatusCode, String)> {
        tracing::debug!("{:?}", sign_in_form);
        let mut conn = state.pool.get().await.map_err(internal_error)?;
        let usr_data: (String, i32) = users::table.select((users::password, users::id)).filter(users::email.eq(sign_in_form.email)).first(&mut conn).await.map_err(|_| (StatusCode::UNAUTHORIZED, String::from("Incorrect email or password, please try again")))?;
        let argon2 = Argon2::default();
        if argon2.verify_password(sign_in_form.password.as_bytes(), &PasswordHash::new(&usr_data.0).map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, String::from("Internal Server Error")))?).is_ok() {
            let session = create_session(usr_data.1, &state.pool).await?;
            let jar = jar.add(Cookie::build(("sc-auth-session", session)).http_only(true).same_site(SameSite::Lax).max_age(Duration::days(30)).path("/"));
            return Ok((AppendHeaders([("HX-Redirect", "/")]), jar))
        }
        Err((StatusCode::UNAUTHORIZED, String::from("Incorrect email or password, please try again")))
    }
}

pub mod signup {

    use argon2::{password_hash::{rand_core::OsRng, SaltString}, Argon2, PasswordHasher};
    use askama::Template;
    use axum::{extract::State, http::StatusCode, response::{AppendHeaders, Html, IntoResponse}, Form};
    use diesel_async::RunQueryDsl;
    use serde::Deserialize;
    use diesel::{insert_into, ExpressionMethods, QueryDsl};
    use crate::{db::{models::NewUser, schema::users}, internal_error, AppState};

    #[derive(Template)]
    #[template(path="signup.html")]
    struct SignUpPage {

    }

    #[derive(Debug, Deserialize)]
    pub struct SignUpData {
        email: String,
        password: String,
        password2: String
    }

    pub async fn sign_up() -> impl IntoResponse {
        let template = SignUpPage {};
        let html = template.render().unwrap();
        return (StatusCode::OK, Html(html))
    }

    pub async fn process_sign_up(State(state): State<AppState>, Form(sign_up_form): Form<SignUpData>) -> Result<impl IntoResponse, impl IntoResponse> {
        tracing::debug!("{:?}", sign_up_form);
        if sign_up_form.password != sign_up_form.password2 {
            return Err((StatusCode::BAD_REQUEST, String::from("ERROR: Your passwords do not match, please try again")))
        }

        // TODO: Verify email is in correct format

        let mut conn = state.pool.get().await.map_err(internal_error)?;

        let result: Vec<String> = users::table.select(users::email).filter(users::email.eq(&sign_up_form.email)).load(&mut conn).await.map_err(internal_error)?;
        if result.len() > 0 {
            return Err((StatusCode::BAD_REQUEST, String::from("ERROR: Unable to create account, email is already in use")))
        }

        let argon2 = Argon2::default();
        let salt = SaltString::generate(&mut OsRng);
        let hash = argon2.hash_password(sign_up_form.password.as_bytes(), &salt).map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, String::from("Interal Server Error")))?;
        let new_user = NewUser {email: &sign_up_form.email, password: &hash.to_string()};

        let n = insert_into(users::table).values(&new_user).execute(&mut conn).await.map_err(internal_error)?;
        if n > 0 {
            Ok(AppendHeaders([("HX-Redirect", "/")]).into_response())
        } else {
            Err((StatusCode::INTERNAL_SERVER_ERROR, String::from("Interal Server Error")))
        }
    }

}