use crate::admin::data::Admin;
use crate::db::Db;
use crate::response::error::ApiError;
use crate::user::data::{PartialUser, SortDirection, TotalUsers, User, UserOrder};
use rocket_contrib::uuid::Uuid;

use rocket_contrib::json::Json;
use serde::Serialize;
use std::str::FromStr;

#[get("/list?<project>&<order_by>&<sort>&<offset>&<limit>")]
pub async fn handler(
    pool: Db<'_>,
    project: Uuid,
    order_by: String,
    sort: String,
    offset: String,
    limit: String,
    _admin: Admin,
) -> Result<Json<Response>, ApiError> {
    let offset = match offset.as_str().parse::<i64>() {
        Err(_) => return Err(ApiError::InternalServerError),
        Ok(offset) => offset,
    };

    let limit = match limit.as_str().parse::<i64>() {
        Err(_) => return Err(ApiError::InternalServerError),
        Ok(limit) => limit,
    };

    let order_by = match UserOrder::from_str(order_by.as_str()) {
        Err(_) => return Err(ApiError::InternalServerError),
        Ok(order) => order,
    };

    let direction = match SortDirection::from_str(sort.as_str()) {
        Err(_) => return Err(ApiError::InternalServerError),
        Ok(sort) => sort,
    };

    let users = User::list(pool.inner(), &project, &order_by, direction, offset, limit).await?;
    let result = Response { items: users };
    Ok(Json(result))
}

#[derive(Serialize)]
pub struct Response {
    pub items: Vec<PartialUser>,
}

#[get("/total?<project>")]
pub async fn total(pool: Db<'_>, project: Uuid) -> Result<Json<TotalUsers>, ApiError> {
    let result = User::total(pool.inner(), &project).await?;
    Ok(Json(result))
}
