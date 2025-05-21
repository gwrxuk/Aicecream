use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::trace::TraceLayer;

use crate::{
    config::Config,
    infra::Controller,
    monitoring::System,
    rollup::{Manager, RollupConfig, RollupStatus},
};

pub struct AppState {
    config: Arc<Config>,
    infra_controller: Arc<Controller>,
    rollup_manager: Arc<Manager>,
    monitoring: Arc<System>,
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/metrics", get(metrics))
        .route("/api/v1/rollups", post(create_rollup))
        .route("/api/v1/rollups/:id", get(get_rollup))
        .route("/api/v1/rollups/:id", post(delete_rollup))
        .route("/api/v1/clusters", post(create_cluster))
        .route("/api/v1/clusters/:id", get(get_cluster))
        .route("/api/v1/clusters/:id", post(delete_cluster))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}

async fn metrics(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    state.monitoring.get_metrics().await
}

#[derive(Debug, Deserialize)]
struct CreateRollupRequest {
    config: RollupConfig,
}

async fn create_rollup(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateRollupRequest>,
) -> impl IntoResponse {
    match state.rollup_manager.create_rollup(req.config).await {
        Ok(_) => StatusCode::CREATED.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn get_rollup(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.rollup_manager.get_rollup_status(&id).await {
        Ok(Some(status)) => Json(status).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn delete_rollup(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.rollup_manager.delete_rollup(&id).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

#[derive(Debug, Deserialize)]
struct CreateClusterRequest {
    name: String,
    region: String,
    node_count: i32,
    node_type: String,
    kubernetes_version: String,
}

async fn create_cluster(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateClusterRequest>,
) -> impl IntoResponse {
    let config = crate::infra::ClusterConfig {
        name: req.name,
        region: req.region,
        node_count: req.node_count,
        node_type: req.node_type,
        kubernetes_version: req.kubernetes_version,
        tags: std::collections::HashMap::new(),
    };

    match state.infra_controller.create_cluster(config).await {
        Ok(_) => StatusCode::CREATED.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn get_cluster(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.infra_controller.get_cluster_status(&id).await {
        Ok(status) => Json(status).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn delete_cluster(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.infra_controller.delete_cluster(&id).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
} 