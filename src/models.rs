// src/models.rs
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct NamespaceInfo {
    pub name: String,
    pub status: String,
    pub created_at: Option<String>,
    pub labels: BTreeMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PodInfo {
    pub name: String,
    pub namespace: String,
    pub phase: String,
    pub ready: String,
    pub restarts: i32,
    pub created_at: Option<String>,
    pub node_name: Option<String>,
    pub labels: BTreeMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub namespace: String,
    pub service_type: String,
    pub cluster_ip: String,
    pub external_ip: String,
    pub ports: String,
    pub created_at: Option<String>,
    pub labels: BTreeMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeploymentInfo {
    pub name: String,
    pub namespace: String,
    pub ready_replicas: i32,
    pub replicas: i32,
    pub updated_replicas: i32,
    pub available_replicas: i32,
    pub created_at: Option<String>,
    pub labels: BTreeMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigMapInfo {
    pub name: String,
    pub namespace: String,
    pub data_keys: Vec<String>,
    pub created_at: Option<String>,
    pub labels: BTreeMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkPolicyInfo {
    pub name: String,
    pub namespace: String,
    pub pod_selector: BTreeMap<String, String>,
    pub ingress_rules: usize,
    pub egress_rules: usize,
    pub created_at: Option<String>,
    pub labels: BTreeMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContainerInfo {
    pub name: String,
    pub image: String,
    pub ports: Vec<String>,
    pub resources: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PodDetails {
    pub name: String,
    pub namespace: String,
    pub labels: BTreeMap<String, String>,
    pub annotations: BTreeMap<String, String>,
    pub node_name: String,
    pub status: String,
    pub containers: Vec<ContainerInfo>,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PodLogs {
    pub pod_name: String,
    pub namespace: String,
    pub logs: String,
}