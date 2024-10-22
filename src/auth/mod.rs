pub mod signin {

    use argon2::{Argon2, PasswordHash, PasswordVerifier};
    use askama::Template;
    use axum::{extract::State, http::StatusCode, response::{AppendHeaders, Html, IntoResponse}, Form};
    use diesel::{ExpressionMethods, QueryDsl};
    use diesel_async::RunQueryDsl;
    use serde::Deserialize;

    use crate::{db::schema::users, internal_error, AppState};

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

    pub async fn process_sign_in(state: State<AppState>, Form(sign_in_form): Form<SignInData>) -> Result<impl IntoResponse, impl IntoResponse> {
        tracing::debug!("{:?}", sign_in_form);
        let mut conn = state.pool.get().await.map_err(internal_error)?;
        let usrs: Vec<String> = users::table.select(users::password).filter(users::email.eq(sign_in_form.email)).limit(1).load(&mut conn).await.map_err(internal_error)?;
        if usrs.len() == 1 {
            let argon2 = Argon2::default();
            if argon2.verify_password(sign_in_form.password.as_bytes(), &PasswordHash::new(&usrs[0]).map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, String::from("Internal Server Error")))?).is_ok() {
                // TODO create a session in database so they stay signed in, add session cookie
                return Ok(AppendHeaders([("HX-Redirect", "/")]).into_response())

            }
        }
        Err((StatusCode::UNAUTHORIZED, String::from("Incorrect Email or Password")))



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
            return Err((StatusCode::BAD_REQUEST, String::from("Your passwords do not match")))
        }

        // TODO: Verify email is in correct format

        let mut conn = state.pool.get().await.map_err(internal_error)?;

        let result: Vec<String> = users::table.select(users::email).filter(users::email.eq(&sign_up_form.email)).load(&mut conn).await.map_err(internal_error)?;
        if result.len() > 0 {
            return Err((StatusCode::BAD_REQUEST, String::from("Unable to create account, email is already in use")))
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