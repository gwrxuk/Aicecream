mod cloud;
mod kubernetes;

use anyhow::Result;
use async_trait::async_trait;
use kube::Client;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::Config;

pub use cloud::CloudProvider;
pub use kubernetes::KubernetesManager;

#[async_trait]
pub trait InfrastructureProvider: Send + Sync {
    async fn create_cluster(&self, name: &str, config: &ClusterConfig) -> Result<()>;
    async fn delete_cluster(&self, name: &str) -> Result<()>;
    async fn get_cluster_status(&self, name: &str) -> Result<ClusterStatus>;
    async fn scale_cluster(&self, name: &str, node_count: i32) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct ClusterConfig {
    pub name: String,
    pub region: String,
    pub node_count: i32,
    pub node_type: String,
    pub kubernetes_version: String,
    pub tags: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct ClusterStatus {
    pub name: String,
    pub state: ClusterState,
    pub node_count: i32,
    pub version: String,
    pub endpoint: String,
}

#[derive(Debug, Clone)]
pub enum ClusterState {
    Creating,
    Running,
    Scaling,
    Failed,
    Deleting,
}

#[derive(Clone)]
pub struct Controller {
    config: Arc<Config>,
    kubernetes: Arc<KubernetesManager>,
    cloud_provider: Arc<dyn InfrastructureProvider>,
}

impl Controller {
    pub async fn new(config: &Config) -> Result<Self> {
        let kubernetes = KubernetesManager::new(config).await?;
        let cloud_provider = match config.cloud.provider {
            CloudProvider::Aws => Arc::new(cloud::AwsProvider::new(config)?),
            CloudProvider::Gcp => Arc::new(cloud::GcpProvider::new(config)?),
            CloudProvider::Azure => Arc::new(cloud::AzureProvider::new(config)?),
        };

        Ok(Self {
            config: Arc::new(config.clone()),
            kubernetes: Arc::new(kubernetes),
            cloud_provider,
        })
    }

    pub async fn create_cluster(&self, config: ClusterConfig) -> Result<()> {
        self.cloud_provider.create_cluster(&config.name, &config).await?;
        Ok(())
    }

    pub async fn delete_cluster(&self, name: &str) -> Result<()> {
        self.cloud_provider.delete_cluster(name).await?;
        Ok(())
    }

    pub async fn get_cluster_status(&self, name: &str) -> Result<ClusterStatus> {
        self.cloud_provider.get_cluster_status(name).await
    }

    pub async fn scale_cluster(&self, name: &str, node_count: i32) -> Result<()> {
        self.cloud_provider.scale_cluster(name, node_count).await?;
        Ok(())
    }

    pub async fn deploy_application(&self, name: &str, manifest: &str) -> Result<()> {
        self.kubernetes.apply_manifest(name, manifest).await?;
        Ok(())
    }

    pub async fn delete_application(&self, name: &str) -> Result<()> {
        self.kubernetes.delete_manifest(name).await?;
        Ok(())
    }

    pub async fn get_application_status(&self, name: &str) -> Result<String> {
        self.kubernetes.get_manifest_status(name).await
    }
} 