use argon2::password_hash::rand_core::{OsRng, RngCore};
use axum::http::StatusCode;
use diesel::{dsl::{delete, insert_into}, ExpressionMethods, QueryDsl};
use diesel_async::{pooled_connection::deadpool::Pool, AsyncMysqlConnection, RunQueryDsl};
use sha2::Digest;
use time::Duration;

use crate::{db::{models::Session, schema::sessions}, internal_error};




pub async fn create_session(user_id: i32, pool: &Pool<AsyncMysqlConnection>) -> Result<String, (StatusCode, String)>{
    let mut conn = pool.get().await.map_err(internal_error)?;
    let mut hasher = sha2::Sha256::new();
    let token = generate_session_token();
    hasher.update(token.as_bytes());
    let id = hex::encode(hasher.finalize());
    let expires_at = time::OffsetDateTime::now_utc().checked_add(Duration::days(30));
    if expires_at.is_none() {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, String::from("Internal Server Error")))
    }
    let session = Session {
        id, 
        user_id,
        expires_at: expires_at.unwrap()
    };
    let n = insert_into(sessions::table).values(session).execute(&mut conn).await.map_err(internal_error)?;
    if n == 0 {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, String::from("Internal Server Error"))) 
    }

    Ok(token)
}

pub async fn validate_session(token: String, pool: &Pool<AsyncMysqlConnection>) -> Result<Session, (StatusCode, String)> {
    let mut conn = pool.get().await.map_err(internal_error)?;
    let mut hasher = sha2::Sha256::new();
    hasher.update(token.as_bytes());
    let session_id = hex::encode(hasher.finalize());
    // "SELECT user_session.id, user_session.user_id, user_session.expires_at, user.id FROM user_session INNER JOIN user ON user.id = user_session.user_id WHERE id = ?"
    let session: Session = sessions::table.select(sessions::all_columns).filter(sessions::id.eq(&session_id)).first(&mut conn).await.map_err(|_| (StatusCode::UNAUTHORIZED, String::from("401 Unauthorized, please try signing out and back in again")))?;
    if time::OffsetDateTime::now_utc() > session.expires_at {
        delete(sessions::table).filter(sessions::id.eq(session_id)).execute(&mut conn).await.map_err(internal_error)?;
    }
    Ok(session)
}


fn generate_session_token() -> String {
    let mut bytes = [0; 20];
    OsRng.fill_bytes(&mut bytes);
    base32::encode(base32::Alphabet::Rfc4648Lower { padding: true }, &bytes)
}

