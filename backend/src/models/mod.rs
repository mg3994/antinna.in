// use chrono::{DateTime, Utc};
use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::prelude::*;
use uuid::Uuid;


use salvo::oapi::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, Type, ToSchema)]
#[sqlx(type_name = "gender_enum", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum Gender {
    Male,
    Female,
    NonBinary,
    Transgender,
    Intersex,
    PreferNotToSay,
    Other,
}

#[derive(FromRow, Serialize, Deserialize, Extractible, Debug)]
#[salvo(extract(default_source(from = "body", parse = "json")))]
pub struct User {
    #[salvo(extract(source(from = "param")))]
    pub id: Uuid,
    pub firebase_uid: String,

    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub gender: Option<Gender>,
    pub dob: Option<chrono::NaiveDate>,

    pub embedding_dirty: Option<bool>,
    // pub created_at: DateTime<Utc>,
    // pub updated_at: DateTime<Utc>,
}

#[derive(FromRow, Serialize, ToSchema, Debug)]
pub struct SafeUser {
    pub id: Uuid,
    pub firebase_uid: String,

    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub gender: Option<Gender>,
    pub dob: Option<chrono::NaiveDate>,

    pub embedding_dirty: Option<bool>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct FullUserRecord {
    pub id: Uuid,
    pub firebase_uid: String,
    pub display_name: Option<String>,
    pub username: Option<String>, // Added from usernames table
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub gender: Option<Gender>, // Your custom Enum
    pub dob: Option<chrono::NaiveDate>,
    pub embedding_dirty: Option<bool>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
}




#[derive(Debug)]
pub struct ProviderType {
    pub slug: String,
    pub name: String,
}
//
pub struct FirebaseFeatures {
    pub auth: bool,
    pub messaging: bool,
}

impl Default for FirebaseFeatures {
    fn default() -> Self {
        Self { auth: true, messaging: true } // Both by default
    }
}