// src/interface/http/mod.rs


use salvo::prelude::*;
use salvo::oapi::ToSchema;
use serde::Serialize;
use crate::core::errors::AppError;


/// The standard Result type for Core/Domain logic
pub type AppResult<T> = Result<T, AppError>;

pub type JsonResult<T> = Result<Json<T>, AppError>;
pub type EmptyResult = Result<Json<Empty>, AppError>;

#[derive(Serialize, ToSchema, Clone, Copy, Debug)]
pub struct Empty {}

pub fn json_ok<T>(data: T) -> JsonResult<T> {
    Ok(Json(data))
}

pub fn empty_ok() -> JsonResult<Empty> {
    Ok(Json(Empty {}))
}