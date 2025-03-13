use axum::serve;
use casbin::{CoreApi, Enforcer};
use sqlx::postgres::PgPoolOptions;
use std::{env::var, sync::Arc};
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{EnvFilter, FmtSubscriber};
use treasury::{AUTH_MODEL_PATH, AUTH_POLICY_PATH, api::ApiV1};

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to initialize tracing subscriber.");
    let model_path: &'static str = AUTH_MODEL_PATH.get_or_init(|| {
        var("AUTH_MODEL_PATH").expect("Failed to read `AUTH_MODEL_PATH` env variable")
    });

    let policies_path: &'static str = AUTH_POLICY_PATH.get_or_init(|| {
        var("AUTH_POLICY_PATH").expect("Failed to read `AUTH_POLICY_PATH` env variable")
    });

    let enforcer = Arc::new(
        Enforcer::new(model_path, policies_path)
            .await
            .expect("Failed to load authorization policy"),
    );

    let database_url = var("DATABASE_URL").expect("Failed to read `DATABASE_URL` env variable");
    let pool = Arc::new(
        PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to database."),
    );

    info!("Connected to database");

    let listener = TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("Failed to create listener.");

    info!("Listening for traffic at `0.0.0.0:8080`");

    serve(listener, ApiV1::router(pool, enforcer))
        .await
        .expect("Failed to serve app");
}
