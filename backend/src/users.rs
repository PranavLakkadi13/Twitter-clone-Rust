use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Deserialize, Serialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub bio: String,
    pub avatar_url: Option<String>,
    pub banner_url: Option<String>,
    pub created_at: OffsetDateTime,
}

// this is to create a new user
#[derive(Deserialize, Debug)]
pub struct CreateUser {
    pub username: String,
    pub display_name: String,
}

pub async fn get_all(pool: &PgPool) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as!(User, "SELECT * FROM users ORDER BY username")
        .fetch_all(pool)
        .await
}

pub async fn get_by_username(pool: &PgPool, username: String) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as!(User, "SELECT * FROM users WHERE username = $1", username)
        .fetch_optional(pool)
        .await
}

pub async fn create_user(pool: &PgPool, user: CreateUser) -> Result<User, sqlx::Error> {
    sqlx::query_as!(
        User,
        "INSERT INTO users (username, display_name) VALUES($1,$2) RETURNING *",
        user.username,
        user.display_name
    )
    .fetch_one(pool)
    .await
}
