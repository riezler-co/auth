use crate::response::error::ApiError;
use crate::session::data::SessionClaims;

use chrono::NaiveDateTime;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug)]
pub struct Token;

impl Token {
    pub async fn is_valid(
        pool: &PgPool,
        claims: &SessionClaims,
        session: &Uuid,
    ) -> Result<bool, ApiError> {
        let exp = NaiveDateTime::from_timestamp(claims.exp.into(), 0);
        let row = sqlx::query!(
            r#"
                with add_token as (
                    insert into tokens(id, session_id, expire_at)
                    values($1, $2, $3)
                    on conflict(id) do nothing
                    returning id
                )
                select count(add_token.id) = 1 as is_valid
                  from add_token
            "#,
            claims.jti,
            session,
            exp
        )
        .fetch_one(pool)
        .await
        .map_err(|_| ApiError::InternalServerError)?;

        match row.is_valid {
            None => Ok(false),
            Some(value) => Ok(value),
        }
    }

    pub fn create() -> String {
        let mut rng = thread_rng();
        (&mut rng)
            .sample_iter(Alphanumeric)
            .take(32)
            .map(char::from)
            .collect()
    }

    pub fn hash(token: &String) -> Result<String, ApiError> {
        match bcrypt::hash(token.clone(), bcrypt::DEFAULT_COST) {
            Err(_) => Err(ApiError::InternalServerError),
            Ok(hashed) => Ok(hashed),
        }
    }

    pub fn verify(token: &str, compare: &str) -> Result<bool, ApiError> {
        match bcrypt::verify(token, compare) {
            Err(_) => Err(ApiError::InternalServerError),
            Ok(valid) => Ok(valid),
        }
    }
}
