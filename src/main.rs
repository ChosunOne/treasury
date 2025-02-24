use axum::serve;
use casbin::{CoreApi, Enforcer};
use log::info;
use sqlx::postgres::PgPoolOptions;
use std::{
    env::var,
    sync::{Arc, OnceLock},
};
use tokio::{net::TcpListener, sync::RwLock};
use treasury::api::ApiV1;

static AUTH_MODEL_PATH: OnceLock<String> = OnceLock::new();
static AUTH_POLICY_PATH: OnceLock<String> = OnceLock::new();

#[tokio::main]
async fn main() {
    env_logger::init();
    let model_path: &'static str = AUTH_MODEL_PATH.get_or_init(|| {
        var("AUTH_MODEL_PATH").expect("Failed to read `AUTH_MODEL_PATH` env variable")
    });

    let policies_path: &'static str = AUTH_POLICY_PATH.get_or_init(|| {
        var("AUTH_POLICY_PATH").expect("Failed to read `AUTH_POLICY_PATH` env variable")
    });

    let enforcer = Arc::new(RwLock::new(
        Enforcer::new(model_path, policies_path)
            .await
            .expect("Failed to load authorization policy"),
    ));

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

    serve(listener, ApiV1::router(pool, enforcer))
        .await
        .expect("Failed to serve app");
}
