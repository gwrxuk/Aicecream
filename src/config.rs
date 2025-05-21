use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub kubernetes: KubernetesConfig,
    pub monitoring: MonitoringConfig,
    pub cloud: CloudConfig,
    pub rollup: RollupConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
    pub workers: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KubernetesConfig {
    pub context: String,
    pub namespace: String,
    pub in_cluster: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub prometheus_endpoint: String,
    pub grafana_endpoint: String,
    pub alertmanager_endpoint: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CloudConfig {
    pub provider: CloudProvider,
    pub region: String,
    pub credentials_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CloudProvider {
    Aws,
    Gcp,
    Azure,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RollupConfig {
    pub default_chain_id: u64,
    pub sequencer_url: String,
    pub validator_url: String,
    pub batch_submitter_url: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config.yaml".to_string());
        let config_str = std::fs::read_to_string(Path::new(&config_path))?;
        let config: Config = serde_yaml::from_str(&config_str)?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<()> {
        // Validate server config
        if self.server.port == 0 {
            anyhow::bail!("Server port cannot be 0");
        }

        // Validate database config
        if self.database.max_connections < self.database.min_connections {
            anyhow::bail!("Max connections cannot be less than min connections");
        }

        // Validate cloud config
        if !Path::new(&self.cloud.credentials_path).exists() {
            anyhow::bail!("Cloud credentials file does not exist");
        }

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                port: 8080,
                host: "0.0.0.0".to_string(),
                workers: num_cpus::get(),
            },
            database: DatabaseConfig {
                url: "postgres://postgres:postgres@localhost:5432/galato".to_string(),
                max_connections: 20,
                min_connections: 5,
            },
            kubernetes: KubernetesConfig {
                context: "default".to_string(),
                namespace: "galato".to_string(),
                in_cluster: false,
            },
            monitoring: MonitoringConfig {
                prometheus_endpoint: "http://localhost:9090".to_string(),
                grafana_endpoint: "http://localhost:3000".to_string(),
                alertmanager_endpoint: "http://localhost:9093".to_string(),
            },
            cloud: CloudConfig {
                provider: CloudProvider::Aws,
                region: "us-west-2".to_string(),
                credentials_path: "~/.aws/credentials".to_string(),
            },
            rollup: RollupConfig {
                default_chain_id: 1337,
                sequencer_url: "http://localhost:8545".to_string(),
                validator_url: "http://localhost:8546".to_string(),
                batch_submitter_url: "http://localhost:8547".to_string(),
            },
        }
    }
} 