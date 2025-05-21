use anyhow::Result;
use async_trait::async_trait;
use ethers::{
    providers::{Http, Provider, Ws},
    types::{Address, U256},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{
    config::Config,
    infra::Controller,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollupConfig {
    pub name: String,
    pub chain_id: u64,
    pub sequencer_address: Address,
    pub validator_address: Address,
    pub batch_submitter_address: Address,
    pub l1_chain_id: u64,
    pub l1_rpc_url: String,
    pub l2_rpc_url: String,
    pub deployment_type: DeploymentType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentType {
    Optimistic,
    ZkRollup,
    Validium,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollupStatus {
    pub name: String,
    pub state: RollupState,
    pub chain_id: u64,
    pub sequencer_status: SequencerStatus,
    pub validator_status: ValidatorStatus,
    pub batch_submitter_status: BatchSubmitterStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RollupState {
    Creating,
    Running,
    Failed,
    Deleting,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequencerStatus {
    pub is_healthy: bool,
    pub last_block: u64,
    pub last_timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorStatus {
    pub is_healthy: bool,
    pub last_validated_block: u64,
    pub last_validation_timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchSubmitterStatus {
    pub is_healthy: bool,
    pub last_submitted_batch: u64,
    pub last_submission_timestamp: u64,
}

pub struct Manager {
    config: Arc<Config>,
    infra_controller: Arc<Controller>,
    rollups: Arc<RwLock<Vec<RollupStatus>>>,
}

impl Manager {
    pub async fn new(config: &Config, infra_controller: Arc<Controller>) -> Result<Self> {
        Ok(Self {
            config: Arc::new(config.clone()),
            infra_controller,
            rollups: Arc::new(RwLock::new(Vec::new())),
        })
    }

    pub async fn create_rollup(&self, config: RollupConfig) -> Result<()> {
        // Deploy Kubernetes resources for the rollup
        let manifest = self.generate_rollup_manifest(&config)?;
        self.infra_controller.deploy_application(&config.name, &manifest).await?;

        // Initialize rollup status
        let status = RollupStatus {
            name: config.name.clone(),
            state: RollupState::Creating,
            chain_id: config.chain_id,
            sequencer_status: SequencerStatus {
                is_healthy: false,
                last_block: 0,
                last_timestamp: 0,
            },
            validator_status: ValidatorStatus {
                is_healthy: false,
                last_validated_block: 0,
                last_validation_timestamp: 0,
            },
            batch_submitter_status: BatchSubmitterStatus {
                is_healthy: false,
                last_submitted_batch: 0,
                last_submission_timestamp: 0,
            },
        };

        let mut rollups = self.rollups.write().await;
        rollups.push(status);

        Ok(())
    }

    pub async fn delete_rollup(&self, name: &str) -> Result<()> {
        // Delete Kubernetes resources
        self.infra_controller.delete_application(name).await?;

        // Remove from rollups list
        let mut rollups = self.rollups.write().await;
        rollups.retain(|r| r.name != name);

        Ok(())
    }

    pub async fn get_rollup_status(&self, name: &str) -> Result<Option<RollupStatus>> {
        let rollups = self.rollups.read().await;
        Ok(rollups.iter().find(|r| r.name == name).cloned())
    }

    pub async fn list_rollups(&self) -> Result<Vec<RollupStatus>> {
        let rollups = self.rollups.read().await;
        Ok(rollups.clone())
    }

    fn generate_rollup_manifest(&self, config: &RollupConfig) -> Result<String> {
        // Generate Kubernetes manifest for the rollup deployment
        let manifest = format!(
            r#"apiVersion: apps/v1
kind: Deployment
metadata:
  name: {}-sequencer
  namespace: {}
spec:
  replicas: 1
  selector:
    matchLabels:
      app: {}-sequencer
  template:
    metadata:
      labels:
        app: {}-sequencer
    spec:
      containers:
      - name: sequencer
        image: galato/sequencer:latest
        env:
        - name: CHAIN_ID
          value: "{}"
        - name: SEQUENCER_ADDRESS
          value: "{}"
        - name: L1_RPC_URL
          value: "{}"
        - name: L2_RPC_URL
          value: "{}"
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: {}-validator
  namespace: {}
spec:
  replicas: 1
  selector:
    matchLabels:
      app: {}-validator
  template:
    metadata:
      labels:
        app: {}-validator
    spec:
      containers:
      - name: validator
        image: galato/validator:latest
        env:
        - name: CHAIN_ID
          value: "{}"
        - name: VALIDATOR_ADDRESS
          value: "{}"
        - name: L1_RPC_URL
          value: "{}"
        - name: L2_RPC_URL
          value: "{}"
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: {}-batch-submitter
  namespace: {}
spec:
  replicas: 1
  selector:
    matchLabels:
      app: {}-batch-submitter
  template:
    metadata:
      labels:
        app: {}-batch-submitter
    spec:
      containers:
      - name: batch-submitter
        image: galato/batch-submitter:latest
        env:
        - name: CHAIN_ID
          value: "{}"
        - name: BATCH_SUBMITTER_ADDRESS
          value: "{}"
        - name: L1_RPC_URL
          value: "{}"
        - name: L2_RPC_URL
          value: "{}"
"#,
            config.name,
            self.config.kubernetes.namespace,
            config.name,
            config.name,
            config.chain_id,
            config.sequencer_address,
            config.l1_rpc_url,
            config.l2_rpc_url,
            config.name,
            self.config.kubernetes.namespace,
            config.name,
            config.name,
            config.chain_id,
            config.validator_address,
            config.l1_rpc_url,
            config.l2_rpc_url,
            config.name,
            self.config.kubernetes.namespace,
            config.name,
            config.name,
            config.chain_id,
            config.batch_submitter_address,
            config.l1_rpc_url,
            config.l2_rpc_url,
        );

        Ok(manifest)
    }
} 