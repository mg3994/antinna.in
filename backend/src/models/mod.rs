// use chrono::{DateTime, Utc};
use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::prelude::*;
use uuid::Uuid;

#[derive(FromRow, Serialize, Deserialize, Extractible, Debug)]
#[salvo(extract(default_source(from = "body", parse = "json")))]
pub struct User {
    #[salvo(extract(source(from = "param")))]
    pub id: Uuid,
    pub username: String,
    pub password: String,
    // pub created_at: DateTime<Utc>,
    // pub updated_at: DateTime<Utc>,
}

#[derive(FromRow, Serialize, ToSchema, Debug)]
pub struct SafeUser {
    pub id: Uuid,
    pub username: String,
}
