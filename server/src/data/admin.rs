use crate::data::token::Claims;
use crate::data::user::User;
use crate::data::{get_query, GenericClient};
use crate::response::error::ApiError;

use bcrypt::{hash, DEFAULT_COST};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use rocket::http::Status;
use rocket::request::Outcome;
use rocket::request::{self, FromRequest, Request};
use rocket_contrib::databases::postgres::error::DbError;
use rocket_contrib::databases::postgres::error::SqlState;
use serde::Deserialize;
use serde_json::json;
use serde_json::value::Value;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct NewProject {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct NewAdmin {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct NewUser {
    pub email: String,
    pub project_id: Uuid,
    pub password: Option<String>,
    pub display_name: Option<String>,
    pub data: Option<Value>,
    pub provider_id: Option<String>,
}

#[derive(Debug)]
pub struct Admin(User);

impl Admin {
    pub fn create<C: GenericClient>(
        client: &mut C,
        body: NewAdmin,
        project: Uuid,
    ) -> Result<Uuid, ApiError> {
        let query = get_query("admin/create")?;

        let password = match hash(body.password.clone(), DEFAULT_COST) {
            Err(_) => return Err(ApiError::InternalServerError),
            Ok(hashed) => hashed,
        };

        let row = client.query_one(query, &[&body.email, &password, &project]);

        match row {
            Err(err) => {
                if let Some(db_error) = err.into_source().unwrap().downcast_ref::<DbError>() {
                    return match db_error.constraint() {
                        Some("users_project_id_fkey") => Err(ApiError::ProjectNotFound),
                        Some("users_project_id_email_key") => Err(ApiError::AdminExists),
                        _ => Err(ApiError::InternalServerError),
                    };
                }

                Err(ApiError::InternalServerError)
            }
            Ok(row) => Ok(row.get("id")),
        }
    }

    pub fn has_admin<C: GenericClient>(client: &mut C) -> Result<bool, ApiError> {
        let query = get_query("admin/has_admin")?;
        let row = client.query_one(query, &[]);

        match row {
            Ok(result) => Ok(result.get("has_admin")),
            Err(_) => Err(ApiError::InternalServerError),
        }
    }

    pub fn create_project<C: GenericClient>(
        client: &mut C,
        body: NewProject,
    ) -> Result<Uuid, ApiError> {
        let query = get_query("project/create")?;

        let name = body.name.clone();
        let row = client.query_one(query, &[&name]);

        match row {
            Err(err) => match err.code() {
                Some(sql_state) => {
                    if sql_state == &SqlState::UNIQUE_VIOLATION {
                        return Err(ApiError::ProjectNameExists);
                    }

                    Err(ApiError::InternalServerError)
                }
                None => Err(ApiError::InternalServerError),
            },
            Ok(row) => Ok(row.get("id")),
        }
    }

    pub fn get_project<C: GenericClient>(client: &mut C) -> Result<Option<Uuid>, ApiError> {
        let query = get_query("admin/has_project")?;
        let rows = client.query(query, &[]);

        match rows {
            Ok(result) => {
                if result.len() == 0 {
                    Ok(None)
                } else {
                    let row = result.get(0).unwrap();
                    Ok(row.get("id"))
                }
            }
            Err(_) => return Err(ApiError::InternalServerError),
        }
    }

    pub fn create_user<C: GenericClient>(client: &mut C, user: NewUser) -> Result<Uuid, ApiError> {
        let query = get_query("user/create")?;

        let password = match user.password {
            Some(ref password) => match hash(password.clone(), DEFAULT_COST) {
                Err(_) => return Err(ApiError::InternalServerError),
                Ok(hashed) => Some(hashed),
            },
            None => None,
        };

        let data = match user.data {
            Some(ref json) => json.clone(),
            None => json!({}),
        };

        let provider = match user.provider_id {
            Some(ref id) => id.clone(),
            None => String::from("email"),
        };

        let row = client.query_one(
            query,
            &[
                &user.email,
                &password,
                &user.display_name,
                &data,
                &provider,
                &user.project_id,
            ],
        );

        match row {
            Err(err) => match err.code() {
                Some(sql_state) => {
                    if sql_state == &SqlState::UNIQUE_VIOLATION {
                        return Err(ApiError::UserExists);
                    }

                    if sql_state == &SqlState::FOREIGN_KEY_VIOLATION {
                        return Err(ApiError::UserInvalidProject);
                    }

                    Err(ApiError::InternalServerError)
                }
                None => Err(ApiError::InternalServerError),
            },
            Ok(row) => Ok(row.get("id")),
        }
    }
}

#[rocket::async_trait]
impl<'a, 'r> FromRequest<'a, 'r> for Admin {
    type Error = ApiError;

    async fn from_request(req: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let token_string = match req.headers().get_one("Authorization") {
            None => return Outcome::Failure((Status::BadRequest, ApiError::AuthTokenMissing)),
            Some(token) => token,
        };

        let end = token_string.len();
        let start = "Bearer ".len();
        let token = &token_string[start..end];

        let token_data = match decode::<Claims>(
            &token,
            &DecodingKey::from_secret("secret".as_ref()),
            &Validation::new(Algorithm::HS256),
        ) {
            Ok(token) => token,
            Err(_) => return Outcome::Failure((Status::BadRequest, ApiError::BadRequest)),
        };

        let user = token_data.claims.user;

        if !user.traits.contains(&String::from("Admin")) {
            return Outcome::Failure((Status::BadRequest, ApiError::AdminAuth));
        }

        Outcome::Success(Admin(user))
    }
}