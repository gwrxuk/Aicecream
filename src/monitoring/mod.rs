use anyhow::Result;
use metrics::{counter, gauge, histogram};
use metrics_exporter_prometheus::PrometheusBuilder;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::Config;

pub struct System {
    config: Arc<Config>,
    metrics: Arc<RwLock<Metrics>>,
}

#[derive(Default)]
struct Metrics {
    cluster_count: i64,
    pod_count: i64,
    rollup_count: i64,
    error_count: i64,
    request_latency: f64,
}

impl System {
    pub async fn new(config: &Config) -> Result<Self> {
        // Initialize Prometheus metrics exporter
        let builder = PrometheusBuilder::new();
        builder
            .with_namespace("galato")
            .with_endpoint(config.monitoring.prometheus_endpoint.clone())
            .install()?;

        Ok(Self {
            config: Arc::new(config.clone()),
            metrics: Arc::new(RwLock::new(Metrics::default())),
        })
    }

    pub async fn record_cluster_created(&self) {
        counter!("clusters_created_total", 1);
        let mut metrics = self.metrics.write().await;
        metrics.cluster_count += 1;
        gauge!("clusters_active", metrics.cluster_count as f64);
    }

    pub async fn record_cluster_deleted(&self) {
        counter!("clusters_deleted_total", 1);
        let mut metrics = self.metrics.write().await;
        metrics.cluster_count -= 1;
        gauge!("clusters_active", metrics.cluster_count as f64);
    }

    pub async fn record_pod_created(&self) {
        counter!("pods_created_total", 1);
        let mut metrics = self.metrics.write().await;
        metrics.pod_count += 1;
        gauge!("pods_active", metrics.pod_count as f64);
    }

    pub async fn record_pod_deleted(&self) {
        counter!("pods_deleted_total", 1);
        let mut metrics = self.metrics.write().await;
        metrics.pod_count -= 1;
        gauge!("pods_active", metrics.pod_count as f64);
    }

    pub async fn record_rollup_created(&self) {
        counter!("rollups_created_total", 1);
        let mut metrics = self.metrics.write().await;
        metrics.rollup_count += 1;
        gauge!("rollups_active", metrics.rollup_count as f64);
    }

    pub async fn record_rollup_deleted(&self) {
        counter!("rollups_deleted_total", 1);
        let mut metrics = self.metrics.write().await;
        metrics.rollup_count -= 1;
        gauge!("rollups_active", metrics.rollup_count as f64);
    }

    pub async fn record_error(&self) {
        counter!("errors_total", 1);
        let mut metrics = self.metrics.write().await;
        metrics.error_count += 1;
        gauge!("errors_active", metrics.error_count as f64);
    }

    pub async fn record_request_latency(&self, latency: f64) {
        histogram!("request_latency_seconds", latency);
        let mut metrics = self.metrics.write().await;
        metrics.request_latency = latency;
        gauge!("request_latency_current", latency);
    }

    pub async fn get_metrics(&self) -> String {
        let metrics = self.metrics.read().await;
        format!(
            "clusters_active: {}\npods_active: {}\nrollups_active: {}\nerrors_active: {}\nrequest_latency: {}",
            metrics.cluster_count,
            metrics.pod_count,
            metrics.rollup_count,
            metrics.error_count,
            metrics.request_latency
        )
    }
} 