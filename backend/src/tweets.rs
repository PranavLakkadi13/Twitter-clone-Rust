use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Deserialize, Serialize, sqlx::FromRow)]
pub struct Tweet {
    pub id: Uuid,
    pub author_id: Uuid,
    pub content: String,
    pub created_at: OffsetDateTime,
    pub author_username: String,
    pub author_display_name: String,
    pub author_avatar_url: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct CreateTweet {
    pub author_id: Uuid,
    pub content: String,
}

pub async fn get_all(pool: &PgPool) -> Result<Vec<Tweet>, sqlx::Error> {
    sqlx::query_as!(
        Tweet,
        r#" SELECT t.id, t.author_id, t.content,t.created_at,
    u.username AS author_username,
    u.display_name AS author_display_name,
    u.avatar_url AS author_avatar_url
    FROM tweets t
    JOIN users u ON u.id = t.author_id
    ORDER BY  t.created_at DESC
    "#
    )
    .fetch_all(pool)
    .await
}

pub async fn get_by_username(pool: &PgPool, username: String) -> Result<Vec<Tweet>, sqlx::Error> {
    sqlx::query_as!(
        Tweet,
        r#" SELECT t.id, t.author_id, t.content,t.created_at,
    u.username AS author_username,
    u.display_name AS author_display_name,
    u.avatar_url AS author_avatar_url 
    FROM tweets t 
    JOIN users u ON u.id = t.author_id 
    WHERE u.username = $1 
    ORDER BY t.created_at DESC"#,
        username
    )
    .fetch_all(pool)
    .await
}

pub async fn get_feed(pool: &PgPool, uuid: Uuid) -> Result<Vec<Tweet>, sqlx::Error> {
    sqlx::query_as!(
        Tweet,
        r#" SELECT t.id, t.author_id, t.content,t.created_at,
    u.username AS author_username,
    u.display_name AS author_display_name,
    u.avatar_url AS author_avatar_url 
    FROM tweets t 
    JOIN users u ON u.id = t.author_id 
    JOIN follows f on f.following_id = t.author_id AND f.follower_id = $1
    ORDER BY t.created_at DESC"#,
        uuid
    )
    .fetch_all(pool)
    .await
}

pub async fn create(pool: &PgPool, user: CreateTweet) -> Result<Tweet, sqlx::Error> {
    sqlx::query_as!(
        Tweet,
        r#" WITH inserted AS (
        INSERT INTO tweets (author_id, content) 
        VALUES ($1, $2) 
        RETURNING *)
        SELECT i.id, i.author_id, i.content, i.created_at, 
        u.username AS author_username,
        u.display_name AS author_display_name,
        u.avatar_url AS author_avatar_url
        FROM inserted i 
        JOIN users u ON u.id = i.author_id
        "#,
        user.author_id,
        user.content
    )
    .fetch_one(pool)
    .await
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<Option<Uuid>, sqlx::Error> {
    let row = sqlx::query_scalar!("DELETE FROM tweets WHERE id = $1 RETURNING id", id)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}
