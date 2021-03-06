use crate::response::error::ApiError;

use bcrypt::{hash, DEFAULT_COST};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json;
use serde_json::value::Value;
use sqlx::{postgres::PgDatabaseError, Error, PgPool};
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct User {
    pub id: Uuid,
    pub display_name: Option<String>,
    pub email: String,
    pub email_verified: bool,
    pub photo_url: Option<String>,
    pub traits: Vec<String>,
    pub data: Value,
    pub provider_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub disabled: bool,
    #[serde(skip_serializing)]
    pub password: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateUser {
    pub id: Uuid,
    pub display_name: Option<String>,
    pub email: String,
    pub traits: Vec<String>,
    pub data: Value,
}

impl User {
    pub async fn update(pool: &PgPool, user: &UpdateUser) -> Result<(), ApiError> {
        sqlx::query!(
            r#"
                update users
                   set display_name = $2
                     , email = $3
                     , traits = $4
                     , data = $5
                 where id = $1
            "#,
            user.id,
            user.display_name,
            user.email,
            &user.traits,
            user.data
        )
        .execute(pool)
        .await
        .map_err(|_| ApiError::InternalServerError)?;

        Ok(())
    }

    pub async fn get_by_email(
        pool: &PgPool,
        email: &str,
        project: Uuid,
    ) -> Result<Option<User>, ApiError> {
        let row = sqlx::query!(
            r#"
                select id
                     , display_name
                     , email
                     , email_verified
                     , photo_url
                     , traits
                     , data
                     , provider_id
                     , created_at
                     , updated_at
                     , disabled
                  from users
                 where email = $1
                   and project_id = $2
            "#,
            email,
            project,
        )
        .fetch_optional(pool)
        .await
        .map_err(|_| ApiError::InternalServerError)?;

        let user = row.map(|u| User {
            id: u.id,
            password: None,
            display_name: u.display_name,
            email: u.email,
            email_verified: u.email_verified,
            photo_url: u.photo_url,
            traits: u.traits,
            data: u.data,
            provider_id: u.provider_id,
            created_at: u.created_at,
            updated_at: u.updated_at,
            disabled: u.disabled,
        });

        Ok(user)
    }

    pub async fn get_by_id(pool: &PgPool, id: &Uuid, project: &Uuid) -> Result<User, ApiError> {
        let row = sqlx::query!(
            r#"
             select id
                  , display_name
                  , email
                  , email_verified
                  , photo_url
                  , traits
                  , data
                  , provider_id
                  , created_at
                  , updated_at
                  , disabled
               from users
              where id = $1
                and project_id = $2
            "#,
            id,
            project,
        )
        .fetch_optional(pool)
        .await
        .map_err(|_| ApiError::InternalServerError)?;

        match row {
            None => Err(ApiError::UserNotFound),
            Some(u) => {
                let user = User {
                    id: u.id,
                    password: None,
                    display_name: u.display_name,
                    email: u.email,
                    email_verified: u.email_verified,
                    photo_url: u.photo_url,
                    traits: u.traits,
                    data: u.data,
                    provider_id: u.provider_id,
                    created_at: u.created_at,
                    updated_at: u.updated_at,
                    disabled: u.disabled,
                };

                Ok(user)
            }
        }
    }

    pub async fn password(pool: &PgPool, email: String, project: &Uuid) -> Result<User, ApiError> {
        let row = sqlx::query_as!(
            User,
            r#"
                select password
                     , id
                     , display_name
                     , email
                     , email_verified
                     , photo_url
                     , traits
                     , data
                     , provider_id
                     , created_at
                     , updated_at
                     , disabled
                  from users
                 where email = $1
                   and project_id = $2
            "#,
            email,
            project,
        )
        .fetch_optional(pool)
        .await
        .map_err(|_| ApiError::InternalServerError)?;

        row.ok_or_else(|| ApiError::UserNotFound)
    }

    pub async fn create(
        pool: &PgPool,
        email: &str,
        password: &str,
        project: Uuid,
    ) -> Result<User, ApiError> {
        let password =
            hash(password.clone(), DEFAULT_COST).map_err(|_| ApiError::InternalServerError)?;

        let row = sqlx::query!(
            r#"
                insert into users
                    ( email
                    , password
                    , project_id
                    , provider_id
                    )
                values
                    ( $1
                    , $2
                    , $3
                    , 'password'
                    )
                returning id
                        , display_name
                        , email
                        , email_verified
                        , photo_url
                        , traits
                        , data
                        , provider_id
                        , created_at
                        , updated_at
                        , disabled
            "#,
            email,
            password,
            project,
        )
        .fetch_one(pool)
        .await
        .map_err(|err| match err {
            Error::Database(err) => {
                let err = err.downcast::<PgDatabaseError>();
                match err.constraint() {
                    Some("users_project_id_fkey") => ApiError::ProjectNotFound,
                    Some("users_project_id_email_key") => ApiError::UserDuplicate,
                    _ => ApiError::InternalServerError,
                }
            }
            _ => ApiError::InternalServerError,
        })?;

        let user = User {
            id: row.id,
            password: None,
            display_name: row.display_name,
            email: row.email,
            email_verified: row.email_verified,
            photo_url: row.photo_url,
            traits: row.traits,
            data: row.data,
            provider_id: row.provider_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
            disabled: row.disabled,
        };

        Ok(user)
    }

    pub async fn create_passwordless(
        pool: &PgPool,
        email: &str,
        project: &Uuid,
    ) -> Result<User, ApiError> {
        let row = sqlx::query!(
            r#"
                        insert into users
                            ( email
                            , project_id
                            , provider_id
                            , email_verified
                            )
                        values
                            ( $1
                            , $2
                            , 'link'
                            , true
                            )
                        returning id
                                , display_name
                                , email
                                , email_verified
                                , photo_url
                                , traits
                                , data
                                , provider_id
                                , created_at
                                , updated_at
                                , disabled
                    "#,
            email,
            project,
        )
        .fetch_one(pool)
        .await
        .map_err(|err| match err {
            Error::Database(err) => {
                let err = err.downcast::<PgDatabaseError>();
                match err.constraint() {
                    Some("users_project_id_fkey") => ApiError::ProjectNotFound,
                    Some("users_project_id_email_key") => ApiError::UserDuplicate,
                    _ => ApiError::InternalServerError,
                }
            }
            _ => ApiError::InternalServerError,
        })?;

        let user = User {
            id: row.id,
            password: None,
            display_name: row.display_name,
            email: row.email,
            email_verified: row.email_verified,
            photo_url: row.photo_url,
            traits: row.traits,
            data: row.data,
            provider_id: row.provider_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
            disabled: row.disabled,
        };

        Ok(user)
    }

    pub async fn set_password(
        pool: &PgPool,
        user_id: &Uuid,
        password: &str,
    ) -> Result<(), ApiError> {
        let password = match hash(password.clone(), DEFAULT_COST) {
            Err(_) => return Err(ApiError::InternalServerError),
            Ok(hashed) => hashed,
        };

        sqlx::query!(
            r#"
                update users
                   set password = $2
                 where id = $1
            "#,
            user_id,
            password,
        )
        .execute(pool)
        .await
        .map_err(|_| ApiError::InternalServerError)?;

        Ok(())
    }

    // todo: direction
    pub async fn list(
        pool: &PgPool,
        project: &Uuid,
        order_by: &UserOrder,
        _direction: SortDirection,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<PartialUser>, ApiError> {
        sqlx::query_as!(
            PartialUser,
            r#"
                select id
                     , email
                     , email_verified
                     , provider_id
                     , created_at
                     , disabled
                  from users
                 where project_id = $1
                 order by case when $2 = 'created_at' then users.created_at end desc,
                          case when $2 = 'email' then users.email end desc
                 offset $3
                 limit $4
            "#,
            project,
            order_by.to_string(),
            offset,
            limit,
        )
        .fetch_all(pool)
        .await
        .map_err(|_| ApiError::InternalServerError)
    }

    pub async fn total(pool: &PgPool, project: &Uuid) -> Result<TotalUsers, ApiError> {
        let row = sqlx::query!(
            r#"
                select count(users.id) as total_users
                  from users
                 where project_id = $1
            "#,
            project
        )
        .fetch_one(pool)
        .await
        .map_err(|_| ApiError::InternalServerError)?;

        match row.total_users {
            None => Err(ApiError::InternalServerError),
            Some(total_users) => Ok(TotalUsers { total_users }),
        }
    }

    pub async fn remove(pool: &PgPool, user_id: &Uuid) -> Result<(), ApiError> {
        sqlx::query!(
            r#"
                delete from users
                 where id = $1
            "#,
            user_id
        )
        .execute(pool)
        .await
        .map_err(|_| ApiError::InternalServerError)?;

        Ok(())
    }

    pub async fn remove_by_token(pool: &PgPool, token_id: &Uuid) -> Result<(), ApiError> {
        sqlx::query!(
            r#"
                delete from users
                 where id in (
                    select sessions.user_id as id
                      from sessions
                     where sessions.id = $1
                )
            "#,
            token_id
        )
        .execute(pool)
        .await
        .map_err(|_| ApiError::InternalServerError)?;

        Ok(())
    }

    pub async fn disable(pool: &PgPool, user: &Uuid, project: &Uuid) -> Result<(), ApiError> {
        sqlx::query!(
            r#"
                with disable_user as (
                    update users
                       set disabled = true
                     where id = $1
                       and project_id = $2
                 returning id
                )
                delete from sessions
                 where user_id in (
                    select id as user_id
                      from disable_user
                 ) 
            "#,
            user,
            project
        )
        .execute(pool)
        .await
        .map_err(|_| ApiError::InternalServerError)?;

        Ok(())
    }

    pub async fn enable(pool: &PgPool, user: &Uuid, project: &Uuid) -> Result<(), ApiError> {
        sqlx::query!(
            r#"
                update users
                   set disabled = false
                 where id = $1
                   and project_id = $2
            "#,
            user,
            project
        )
        .execute(pool)
        .await
        .map_err(|_| ApiError::InternalServerError)?;

        Ok(())
    }
}

pub enum SortDirection {
    Asc,
    Desc,
}

impl ToString for SortDirection {
    fn to_string(&self) -> String {
        match self {
            SortDirection::Asc => String::from("asc"),
            SortDirection::Desc => String::from("desc"),
        }
    }
}

impl FromStr for SortDirection {
    type Err = ParamError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "asc" => Ok(SortDirection::Asc),
            "desc" => Ok(SortDirection::Desc),
            _ => Err(ParamError::InvalidOption),
        }
    }
}

pub enum UserOrder {
    CreatedAt,
    Email,
}

impl ToString for UserOrder {
    fn to_string(&self) -> String {
        match self {
            UserOrder::CreatedAt => String::from("created_at"),
            UserOrder::Email => String::from("email"),
        }
    }
}

impl FromStr for UserOrder {
    type Err = ParamError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "created_at" => Ok(UserOrder::CreatedAt),
            "email" => Ok(UserOrder::Email),
            _ => Err(ParamError::InvalidOption),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PartialUser {
    id: Uuid,
    email: String,
    email_verified: bool,
    provider_id: String,
    created_at: DateTime<Utc>,
    disabled: bool,
}

pub enum ParamError {
    InvalidOption,
}

#[derive(Serialize)]
pub struct TotalUsers {
    pub total_users: i64,
}
