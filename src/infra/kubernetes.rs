use anyhow::Result;
use k8s_openapi::api::core::v1::Pod;
use kube::{
    api::{Api, DeleteParams, PostParams},
    Client, Config,
};
use std::sync::Arc;

use crate::config::Config;

pub struct KubernetesManager {
    client: Client,
    namespace: String,
}

impl KubernetesManager {
    pub async fn new(config: &Config) -> Result<Self> {
        let kube_config = if config.kubernetes.in_cluster {
            Config::incluster()?
        } else {
            Config::from_kubeconfig(&kube::config::KubeConfigOptions {
                context: Some(config.kubernetes.context.clone()),
                ..Default::default()
            })
            .await?
        };

        let client = Client::try_from(kube_config)?;

        Ok(Self {
            client,
            namespace: config.kubernetes.namespace.clone(),
        })
    }

    pub async fn apply_manifest(&self, name: &str, manifest: &str) -> Result<()> {
        // Parse the manifest
        let resources = kube::api::DynamicObject::parse_yaml(manifest)?;

        // Apply each resource
        for resource in resources {
            let api: Api<kube::api::DynamicObject> = Api::namespaced(
                self.client.clone(),
                &self.namespace,
            );

            api.create(&PostParams::default(), &resource).await?;
        }

        Ok(())
    }

    pub async fn delete_manifest(&self, name: &str) -> Result<()> {
        let api: Api<kube::api::DynamicObject> = Api::namespaced(
            self.client.clone(),
            &self.namespace,
        );

        api.delete(name, &DeleteParams::default()).await?;

        Ok(())
    }

    pub async fn get_manifest_status(&self, name: &str) -> Result<String> {
        let api: Api<kube::api::DynamicObject> = Api::namespaced(
            self.client.clone(),
            &self.namespace,
        );

        let resource = api.get(name).await?;
        let status = serde_json::to_string_pretty(&resource.status)?;

        Ok(status)
    }

    pub async fn get_pod_logs(&self, pod_name: &str) -> Result<String> {
        let api: Api<Pod> = Api::namespaced(
            self.client.clone(),
            &self.namespace,
        );

        let logs = api.logs(pod_name, &Default::default()).await?;
        Ok(logs)
    }

    pub async fn get_pod_status(&self, pod_name: &str) -> Result<String> {
        let api: Api<Pod> = Api::namespaced(
            self.client.clone(),
            &self.namespace,
        );

        let pod = api.get(pod_name).await?;
        let status = serde_json::to_string_pretty(&pod.status)?;

        Ok(status)
    }

    pub async fn list_pods(&self) -> Result<Vec<Pod>> {
        let api: Api<Pod> = Api::namespaced(
            self.client.clone(),
            &self.namespace,
        );

        let pods = api.list(&Default::default()).await?;
        Ok(pods.items)
    }

    pub async fn delete_pod(&self, pod_name: &str) -> Result<()> {
        let api: Api<Pod> = Api::namespaced(
            self.client.clone(),
            &self.namespace,
        );

        api.delete(pod_name, &DeleteParams::default()).await?;
        Ok(())
    }
} 