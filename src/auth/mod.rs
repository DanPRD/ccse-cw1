
pub mod session;

pub mod signin {

    use argon2::{Argon2, PasswordHash, PasswordVerifier};
    use askama::Template;
    use axum::{extract::State, http::StatusCode, response::{AppendHeaders, Html, IntoResponse, Redirect}, Form};
    use axum_extra::extract::{cookie::{Cookie, SameSite}, CookieJar};
    use diesel::{ExpressionMethods, QueryDsl};
    use diesel_async::RunQueryDsl;
    use serde::Deserialize;
    use ::time::Duration;
    use crate::{auth::session::create_session, db::schema::users, internal_error, logged_in, AppState, SESSION_COOKIE_NAME};
    #[derive(Template)]
    #[template(path="signin.html")]
    struct SignInPage {
        logged_in: bool
    }

    #[derive(Debug, Deserialize)]
    pub struct SignInData {
        email: String,
        password: String
    }

    pub async fn sign_in(jar: CookieJar, State(state): State<AppState>) -> impl IntoResponse {
        if logged_in(&jar, &state.pool).await {
            return Redirect::temporary("/").into_response()
        }
        let template = SignInPage {logged_in: false};
        let html = template.render().unwrap();
        return (StatusCode::OK, Html(html)).into_response()
    }

    pub async fn process_sign_in(state: State<AppState>, jar: CookieJar, Form(sign_in_form): Form<SignInData>) -> Result<impl IntoResponse, (StatusCode, String)> {
        let mut conn = state.pool.get().await.map_err(internal_error)?;
        let usr_data: (String, i32, bool) = users::table.select((users::password, users::id, users::is_admin)).filter(users::email.eq(sign_in_form.email)).first(&mut conn).await.map_err(|_| (StatusCode::UNAUTHORIZED, String::from("Incorrect email or password, please try again")))?;
        let argon2 = Argon2::default();
        if argon2.verify_password(sign_in_form.password.as_bytes(), &PasswordHash::new(&usr_data.0).map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, String::from("Internal Server Error")))?).is_ok() {
            let session = create_session(usr_data.1, &state.pool).await?;
            let jar = jar.add(Cookie::build((SESSION_COOKIE_NAME, session)).http_only(true).same_site(SameSite::Lax).max_age(Duration::days(30)).path("/"));
            if usr_data.2 {
                return Ok((AppendHeaders([("HX-Redirect", "/adminpanel")]), jar))
            } else {
                return Ok((AppendHeaders([("HX-Redirect", "/")]), jar))
            }
        }
        Err((StatusCode::UNAUTHORIZED, String::from("Incorrect email or password, please try again")))
    }
}

pub mod signup {

    use argon2::{password_hash::{rand_core::OsRng, SaltString}, Argon2, PasswordHasher};
    use askama::Template;
    use axum::{extract::State, http::StatusCode, response::{AppendHeaders, Html, IntoResponse, Redirect}, Form};
    use axum_extra::extract::CookieJar;
    use diesel_async::RunQueryDsl;
    use serde::Deserialize;
    use diesel::{insert_into, ExpressionMethods, QueryDsl};
    use crate::{db::{models::NewUser, schema::users}, internal_error, logged_in, AppState};

    #[derive(Template)]
    #[template(path="signup.html")]
    struct SignUpPage {
        logged_in: bool,
    }

    #[derive(Debug, Deserialize)]
    pub struct SignUpData {
        email: String,
        password: String,
        password2: String
    }

    pub async fn sign_up(jar: CookieJar, State(state): State<AppState>) -> impl IntoResponse {
        if logged_in(&jar, &state.pool).await {
            return Redirect::temporary("/").into_response()
        }
        let template = SignUpPage {logged_in: false};
        let html = template.render().unwrap();
        return (StatusCode::OK, Html(html)).into_response()
    }

    pub async fn process_sign_up(State(state): State<AppState>, Form(sign_up_form): Form<SignUpData>) -> Result<impl IntoResponse, impl IntoResponse> {
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
            Ok(AppendHeaders([("HX-Redirect", "/sign-in")]).into_response())
        } else {
            Err((StatusCode::INTERNAL_SERVER_ERROR, String::from("Interal Server Error")))
        }
    }

}

pub mod signout {

    use axum::{extract::State, http::StatusCode, response::{AppendHeaders, IntoResponse}};
    use axum_extra::extract::CookieJar;
    use diesel_async::RunQueryDsl;
    use diesel::{delete, ExpressionMethods};
    use crate::{db::schema::sessions, internal_error, AppState, SESSION_COOKIE_NAME};

    use super::session::validate_session;


    pub async fn sign_out(jar: CookieJar, State(state): State<AppState>) -> Result<impl IntoResponse, (StatusCode, String)>{
        let mut conn = state.pool.get().await.map_err(internal_error)?;
        let session_cookie = jar.get(SESSION_COOKIE_NAME).ok_or((StatusCode::UNAUTHORIZED, String::from("Invalid Session")))?;
        let session = validate_session(session_cookie.value().to_owned(), &state.pool).await?;
        delete(sessions::table).filter(sessions::id.eq(session.id)).execute(&mut conn).await.map_err(internal_error)?;
        let jar = jar.remove(SESSION_COOKIE_NAME);
        return Ok((AppendHeaders([("HX-Redirect", "/")]), jar))
        

    }
}
 