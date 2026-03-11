mod agents;
mod auth;
mod config;
mod db;
mod error;
mod executor;

mod routes;
mod scheduler;
mod streaming;

use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use tokio::sync::{mpsc, Mutex};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use uuid::Uuid;

use anyhow::Context;
use crate::config::Config;
use crate::streaming::TaskProgressStream;

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::PgPool,
    pub config: Config,
    pub executor_tx: mpsc::Sender<Uuid>,
    pub progress_tx: mpsc::Sender<(Uuid, TaskProgressStream)>,
}

impl AsRef<AppState> for AppState {
    fn as_ref(&self) -> &AppState {
        self
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env if present
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "errand_api=info,tower_http=info".into()),
        )
        .init();

    // Load config
    let config = Config::from_env()?;
    tracing::info!("Starting Errand API on {}", config.bind_addr);

    // Create connection pool
    tracing::info!("Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(std::time::Duration::from_secs(10))
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                sqlx::query("SET search_path TO errand, public")
                    .execute(&mut *conn)
                    .await?;
                Ok(())
            })
        })
        .connect(&config.database_url)
        .await
        .context("Failed to connect to database — check DATABASE_URL or DB_HOST/DB_USER/DB_PASSWORD/DB_NAME")?;

    tracing::info!("Connected to database");

    // Create channels for background workers
    let (executor_tx, executor_rx) = mpsc::channel::<Uuid>(256);
    let (progress_tx, progress_rx) = mpsc::channel::<(Uuid, TaskProgressStream)>(64);

    let progress_streams: Arc<Mutex<HashMap<Uuid, TaskProgressStream>>> =
        Arc::new(Mutex::new(HashMap::new()));

    let state = AppState {
        pool: pool.clone(),
        config: config.clone(),
        executor_tx,
        progress_tx,
    };

    // Spawn background scheduler
    let scheduler_pool = pool.clone();
    let scheduler_tx = state.executor_tx.clone();
    tokio::spawn(async move {
        scheduler::run_scheduler(scheduler_pool, scheduler_tx).await;
    });

    // Spawn background executor
    let executor_pool = pool.clone();
    let executor_config = config.clone();
    let executor_streams = progress_streams.clone();
    tokio::spawn(async move {
        executor::run_executor(
            executor_pool,
            executor_config,
            executor_rx,
            executor_streams,
            progress_rx,
        )
        .await;
    });

    // Build router
    let app = Router::new()
        // Health
        .route("/api/health", get(health))
        // Auth
        .route("/api/auth/signup", post(auth::signup))
        .route("/api/auth/login", post(auth::login))
        // Tasks
        .route("/api/tasks", post(routes::tasks::create_task))
        .route("/api/tasks", get(routes::tasks::list_tasks))
        .route("/api/tasks/{id}", get(routes::tasks::get_task))
        .route("/api/tasks/{id}/pause", post(routes::tasks::pause_task))
        .route("/api/tasks/{id}/resume", post(routes::tasks::resume_task))
        .route("/api/tasks/{id}/cancel", post(routes::tasks::cancel_task))
        .route("/api/tasks/{id}/approve", post(routes::tasks::approve_step))
        .route("/api/tasks/{id}/stream", get(routes::tasks::stream_task))
        // Agents
        .route("/api/agents", get(routes::agents::list_agents))
        // Credits
        .route("/api/credits/balance", get(routes::credits::get_balance))
        .route("/api/credits/history", get(routes::credits::get_history))
        .route("/api/credits/redeem", post(routes::credits::redeem_code))
        // Middleware
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state);

    // Serve
    let listener = tokio::net::TcpListener::bind(&config.bind_addr).await?;
    tracing::info!("Listening on {}", config.bind_addr);
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health() -> &'static str {
    "ok"
}
