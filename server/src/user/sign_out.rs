use crate::data::admin::Admin;
use crate::data::AuthDb;
use crate::response::error::ApiError;
use crate::session::{RefreshAccessToken, Session};

use rocket;
use rocket_contrib::json::Json;
use rocket_contrib::uuid::Uuid;

#[post("/sign_out/<session_id>", data = "<rat>")]
pub async fn sign_out(
    conn: AuthDb,
    session_id: Uuid,
    rat: Json<RefreshAccessToken>,
) -> Result<(), ApiError> {
    let session = conn
        .run(move |client| Session::get(client, &session_id))
        .await?;

    let is_valid = Session::validate_token(&session, &rat)?;

    if !is_valid {
        return Err(ApiError::Forbidden);
    }

    conn.run(move |client| Session::delete(client, &session.id))
        .await?;

    Ok(())
}

#[post("/sign_out/<user_id>", rank = 2)]
pub async fn admin_sign_out(conn: AuthDb, user_id: Uuid, _admin: Admin) -> Result<(), ApiError> {
    let user_id = user_id.into_inner();

    conn.run(move |client| Session::delete_by_user(client, &user_id))
        .await?;

    Ok(())
}

#[post("/sign_out_all/<session_id>", data = "<rat>")]
pub async fn sign_out_all(
    conn: AuthDb,
    session_id: Uuid,
    rat: Json<RefreshAccessToken>,
) -> Result<(), ApiError> {
    let session = conn
        .run(move |client| Session::get(client, &session_id))
        .await?;

    let is_valid = Session::validate_token(&session, &rat)?;

    if !is_valid {
        return Err(ApiError::Forbidden);
    }

    conn.run(move |client| Session::delete_all(client, &session.id))
        .await?;

    Ok(())
}
