use axum::serve;
use log::info;
use sqlx::postgres::PgPoolOptions;
use std::env::var;
use tokio::net::TcpListener;
use treasury::api::ApiV1;

#[tokio::main]
async fn main() {
    env_logger::init();

    let database_url = var("DATABASE_URL").expect("Failed to read `DATABASE_URL` env variable");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database.");

    info!("Connected to database");

    let listener = TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("Failed to create listener.");

    info!("Listening for traffic at `0.0.0.0:8080`");

    serve(listener, ApiV1::router(pool))
        .await
        .expect("Failed to serve app");
}
