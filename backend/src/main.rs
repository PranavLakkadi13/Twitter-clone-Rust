use axum::{
    Json, Router,
    body::to_bytes,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use serde_json::map;
use sqlx::PgPool;
use tower_http::cors::CorsLayer;

use crate::users::User;

mod follows;
mod tweets;
mod users;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE URL must be set in env");

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed To connect with DATABASE");

    // it will run if we didnt run migrations manually
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let app = Router::new()
        .route("/users", get(get_users).post(create_user))
        .route("/users/{username}", get(get_user))
        .route("/users/{username}/tweets", get(get_user_tweets))
        .route("/tweets", get(get_tweets).post(create_tweet))
        .route("/tweets/{id}", axum::routing::delete(delete_tweet))
        .route("/tweets/feed/{user_id}", get(get_feed))
        .route("/follows/toggle", post(toggle_follow))
        .layer(CorsLayer::permissive())
        .with_state(pool);

    let addr = "0.0.0.0:3000";
    println!("Server running on https://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, app).await.unwrap();
}

///////////////////
// HANDLER SECTION
///////////////////
// USER HANDLER
///////////////////
async fn get_users(State(pool): State<PgPool>) -> Result<Json<Vec<users::User>>, StatusCode> {
    users::get_all(&pool)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn get_user(
    State(pool): State<PgPool>,
    Path(username): Path<String>,
) -> Result<Json<users::User>, StatusCode> {
    users::get_by_username(&pool, username)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn create_user(
    State(pool): State<PgPool>,
    Json(input): Json<users::CreateUser>,
) -> Result<(StatusCode, Json<User>), StatusCode> {
    users::create_user(&pool, input)
        .await
        .map(|u| (StatusCode::CREATED, Json(u)))
        .map_err(|_| StatusCode::BAD_REQUEST)
}

////////////////
// Tweet Handler
////////////////
async fn get_tweets(State(pool): State<PgPool>) -> Result<Json<Vec<tweets::Tweet>>, StatusCode> {
    tweets::get_all(&pool)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn get_user_tweets(
    State(pool): State<PgPool>,
    Path(username): Path<String>,
) -> Result<Json<Vec<tweets::Tweet>>, StatusCode> {
    tweets::get_by_username(&pool, username)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn get_feed(
    State(pool): State<PgPool>,
    Path(user_id): Path<uuid::Uuid>,
) -> Result<Json<Vec<tweets::Tweet>>, StatusCode> {
    tweets::get_feed(&pool, user_id)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_tweet(
    State(pool): State<PgPool>,
    Json(input): Json<tweets::CreateTweet>,
) -> Result<(StatusCode, Json<tweets::Tweet>), StatusCode> {
    tweets::create(&pool, input)
        .await
        .map(|u| (StatusCode::CREATED, Json(u)))
        .map_err(|_| StatusCode::BAD_REQUEST)
}

async fn delete_tweet(
    State(pool): State<PgPool>,
    Path(id): Path<uuid::Uuid>,
) -> Result<StatusCode, StatusCode> {
    tweets::delete(&pool, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(|_| StatusCode::NO_CONTENT)
        .ok_or(StatusCode::NOT_FOUND)
}

////////////////////
// Follow Handler
////////////////////
async fn toggle_follow(
    State(pool): State<PgPool>,
    Json(input): Json<follows::ToggleFollow>,
) -> Result<Json<follows::FollowResult>, StatusCode> {
    follows::toggle(&pool, input)
        .await
        .map(|is_following| Json(follows::FollowResult { is_following }))
        .map_err(|_| StatusCode::BAD_REQUEST)
}
