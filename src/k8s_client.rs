// src/k8s_client.rs
use anyhow::Result;
use kube::{Client, Config};

pub async fn create_client() -> Result<Client> {
    let config = Config::infer().await?;
    let client = Client::try_from(config)?;
    Ok(client)
}

pub fn format_age(timestamp: Option<String>) -> String {
    timestamp.unwrap_or_else(|| "Unknown".to_string())
}

pub fn format_labels(labels: &std::collections::BTreeMap<String, String>) -> String {
    if labels.is_empty() {
        "<none>".to_string()
    } else {
        labels
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join(",")
    }
}