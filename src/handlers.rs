// src/handlers.rs
use anyhow::Result;
use k8s_openapi::api::core::v1::{Namespace, Pod, Service, ConfigMap};
use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::networking::v1::NetworkPolicy;
use kube::{Api, Client, ResourceExt};
use std::convert::Infallible;

use crate::models::*;

pub async fn get_namespaces(client: Client) -> Result<impl warp::Reply, Infallible> {
    let namespaces: Api<Namespace> = Api::all(client);
    
    match namespaces.list(&Default::default()).await {
        Ok(namespace_list) => {
            let ns_info: Vec<NamespaceInfo> = namespace_list
                .items
                .into_iter()
                .map(|ns| NamespaceInfo {
                    name: ns.name_any(),
                    status: ns.status.as_ref().and_then(|s| s.phase.as_ref()).unwrap_or(&String::new()).clone(),
                    created_at: ns.creation_timestamp().map(|ts| ts.0.to_rfc3339()),
                    labels: ns.labels().clone(),
                })
                .collect();
            
            Ok(warp::reply::json(&ns_info))
        }
        Err(e) => {
            eprintln!("Error fetching namespaces: {}", e);
            Ok(warp::reply::json(&Vec::<NamespaceInfo>::new()))
        }
    }
}

pub async fn get_pods(namespace: String, client: Client) -> Result<impl warp::Reply, Infallible> {
    let pods: Api<Pod> = Api::namespaced(client, &namespace);
    
    match pods.list(&Default::default()).await {
        Ok(pod_list) => {
            let pod_info: Vec<PodInfo> = pod_list
                .items
                .into_iter()
                .map(|pod| {
                    let status = pod.status.as_ref();
                    let phase = status
                        .and_then(|s| s.phase.as_ref())
                        .unwrap_or(&"Unknown".to_string())
                        .clone();
                    
                    let ready_containers = status
                        .and_then(|s| s.container_statuses.as_ref())
                        .map(|cs| cs.iter().filter(|c| c.ready).count())
                        .unwrap_or(0);
                    
                    let total_containers = status
                        .and_then(|s| s.container_statuses.as_ref())
                        .map(|cs| cs.len())
                        .unwrap_or(0);

                    PodInfo {
                        name: pod.name_any(),
                        namespace: pod.namespace().unwrap_or_default(),
                        phase,
                        ready: format!("{}/{}", ready_containers, total_containers),
                        restarts: status
                            .and_then(|s| s.container_statuses.as_ref())
                            .map(|cs| cs.iter().map(|c| c.restart_count).sum::<i32>())
                            .unwrap_or(0),
                        created_at: pod.creation_timestamp().map(|ts| ts.0.to_rfc3339()),
                        node_name: status.and_then(|s| s.host_ip.clone()),
                        labels: pod.labels().clone(),
                    }
                })
                .collect();
            
            Ok(warp::reply::json(&pod_info))
        }
        Err(e) => {
            eprintln!("Error fetching pods: {}", e);
            Ok(warp::reply::json(&Vec::<PodInfo>::new()))
        }
    }
}

pub async fn get_services(namespace: String, client: Client) -> Result<impl warp::Reply, Infallible> {
    let services: Api<Service> = Api::namespaced(client, &namespace);
    
    match services.list(&Default::default()).await {
        Ok(service_list) => {
            let service_info: Vec<ServiceInfo> = service_list
                .items
                .into_iter()
                .map(|svc| {
                    let spec = svc.spec.as_ref();
                    let ports = spec
                        .and_then(|s| s.ports.as_ref())
                        .map(|ports| {
                            ports
                                .iter()
                                .map(|p| {
                                    let target_port = p.target_port.as_ref()
                                        .map(|tp| match tp {
                                            k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::Int(i) => i.to_string(),
                                            k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::String(s) => s.clone(),
                                        })
                                        .unwrap_or_default();
                                    format!("{}:{}", p.port, target_port)
                                })
                                .collect::<Vec<_>>()
                                .join(", ")
                        })
                        .unwrap_or_default();

                    ServiceInfo {
                        name: svc.name_any(),
                        namespace: svc.namespace().unwrap_or_default(),
                        service_type: spec
                            .and_then(|s| s.type_.as_ref())
                            .unwrap_or(&"ClusterIP".to_string())
                            .clone(),
                        cluster_ip: spec
                            .and_then(|s| s.cluster_ip.as_ref())
                            .unwrap_or(&"None".to_string())
                            .clone(),
                        external_ip: spec
                            .and_then(|s| s.external_ips.as_ref())
                            .and_then(|ips| ips.first())
                            .unwrap_or(&"<none>".to_string())
                            .clone(),
                        ports,
                        created_at: svc.creation_timestamp().map(|ts| ts.0.to_rfc3339()),
                        labels: svc.labels().clone(),
                    }
                })
                .collect();
            
            Ok(warp::reply::json(&service_info))
        }
        Err(e) => {
            eprintln!("Error fetching services: {}", e);
            Ok(warp::reply::json(&Vec::<ServiceInfo>::new()))
        }
    }
}

pub async fn get_deployments(namespace: String, client: Client) -> Result<impl warp::Reply, Infallible> {
    let deployments: Api<Deployment> = Api::namespaced(client, &namespace);
    
    match deployments.list(&Default::default()).await {
        Ok(deployment_list) => {
            let deployment_info: Vec<DeploymentInfo> = deployment_list
                .items
                .into_iter()
                .map(|dep| {
                    let status = dep.status.as_ref();
                    let spec = dep.spec.as_ref();

                    DeploymentInfo {
                        name: dep.name_any(),
                        namespace: dep.namespace().unwrap_or_default(),
                        ready_replicas: status
                            .and_then(|s| s.ready_replicas)
                            .unwrap_or(0),
                        replicas: spec
                            .and_then(|s| s.replicas)
                            .unwrap_or(0),
                        updated_replicas: status
                            .and_then(|s| s.updated_replicas)
                            .unwrap_or(0),
                        available_replicas: status
                            .and_then(|s| s.available_replicas)
                            .unwrap_or(0),
                        created_at: dep.creation_timestamp().map(|ts| ts.0.to_rfc3339()),
                        labels: dep.labels().clone(),
                    }
                })
                .collect();
            
            Ok(warp::reply::json(&deployment_info))
        }
        Err(e) => {
            eprintln!("Error fetching deployments: {}", e);
            Ok(warp::reply::json(&Vec::<DeploymentInfo>::new()))
        }
    }
}

pub async fn get_configmaps(namespace: String, client: Client) -> Result<impl warp::Reply, Infallible> {
    let configmaps: Api<ConfigMap> = Api::namespaced(client, &namespace);
    
    match configmaps.list(&Default::default()).await {
        Ok(cm_list) => {
            let cm_info: Vec<ConfigMapInfo> = cm_list
                .items
                .into_iter()
                .map(|cm| {
                    let data_keys = cm.data
                        .as_ref()
                        .map(|d| d.keys().cloned().collect::<Vec<_>>())
                        .unwrap_or_default();

                    ConfigMapInfo {
                        name: cm.name_any(),
                        namespace: cm.namespace().unwrap_or_default(),
                        data_keys,
                        created_at: cm.creation_timestamp().map(|ts| ts.0.to_rfc3339()),
                        labels: cm.labels().clone(),
                    }
                })
                .collect();
            
            Ok(warp::reply::json(&cm_info))
        }
        Err(e) => {
            eprintln!("Error fetching configmaps: {}", e);
            Ok(warp::reply::json(&Vec::<ConfigMapInfo>::new()))
        }
    }
}

pub async fn get_network_policies(namespace: String, client: Client) -> Result<impl warp::Reply, Infallible> {
    let network_policies: Api<NetworkPolicy> = Api::namespaced(client, &namespace);
    
    match network_policies.list(&Default::default()).await {
        Ok(np_list) => {
            let np_info: Vec<NetworkPolicyInfo> = np_list
                .items
                .into_iter()
                .map(|np| {
                    let spec = np.spec.as_ref();
                    
                    NetworkPolicyInfo {
                        name: np.name_any(),
                        namespace: np.namespace().unwrap_or_default(),
                        pod_selector: spec
                            .and_then(|s| s.pod_selector.match_labels.as_ref())
                            .cloned()
                            .unwrap_or_default(),
                        ingress_rules: spec
                            .and_then(|s| s.ingress.as_ref())
                            .map(|i| i.len())
                            .unwrap_or(0),
                        egress_rules: spec
                            .and_then(|s| s.egress.as_ref())
                            .map(|e| e.len())
                            .unwrap_or(0),
                        created_at: np.creation_timestamp().map(|ts| ts.0.to_rfc3339()),
                        labels: np.labels().clone(),
                    }
                })
                .collect();
            
            Ok(warp::reply::json(&np_info))
        }
        Err(e) => {
            eprintln!("Error fetching network policies: {}", e);
            Ok(warp::reply::json(&Vec::<NetworkPolicyInfo>::new()))
        }
    }
}

pub async fn get_pod_details(namespace: String, pod_name: String, client: Client) -> Result<impl warp::Reply, Infallible> {
    let pods: Api<Pod> = Api::namespaced(client, &namespace);
    
    match pods.get(&pod_name).await {
        Ok(pod) => {
            let details = PodDetails {
                name: pod.name_any(),
                namespace: pod.namespace().unwrap_or_default(),
                labels: pod.labels().clone(),
                annotations: pod.annotations().clone(),
                node_name: pod.spec.as_ref()
                    .and_then(|s| s.node_name.clone())
                    .unwrap_or_default(),
                status: pod.status.as_ref()
                    .and_then(|s| s.phase.clone())
                    .unwrap_or_default(),
                containers: pod.spec.as_ref()
                    .map(|s| s.containers.iter()
                        .map(|c| ContainerInfo {
                            name: c.name.clone(),
                            image: c.image.clone().unwrap_or_default(),
                            ports: c.ports.as_ref()
                                .map(|ports| ports.iter()
                                    .map(|p| format!("{}:{}", p.container_port, p.protocol.as_ref().unwrap_or(&"TCP".to_string())))
                                    .collect())
                                .unwrap_or_default(),
                            resources: format!(
                                "CPU: {} / MEM: {}",
                                c.resources.as_ref()
                                    .and_then(|r| r.requests.as_ref())
                                    .and_then(|req| req.get("cpu"))
                                    .map(|v| v.0.clone())
                                    .unwrap_or_default(),
                                c.resources.as_ref()
                                    .and_then(|r| r.requests.as_ref())
                                    .and_then(|req| req.get("memory"))
                                    .map(|v| v.0.clone())
                                    .unwrap_or_default()
                            ),
                        })
                        .collect())
                    .unwrap_or_default(),
                created_at: pod.creation_timestamp().map(|ts| ts.0.to_rfc3339()),
            };
            
            Ok(warp::reply::json(&details))
        }
        Err(e) => {
            eprintln!("Error fetching pod details: {}", e);
            Ok(warp::reply::json(&serde_json::json!({"error": "Pod not found"})))
        }
    }
}

pub async fn get_pod_logs(namespace: String, pod_name: String, client: Client) -> Result<impl warp::Reply, Infallible> {
    let pods: Api<Pod> = Api::namespaced(client, &namespace);
    
    match pods.logs(&pod_name, &Default::default()).await {
        Ok(logs) => {
            let log_response = PodLogs {
                pod_name,
                namespace,
                logs,
            };
            Ok(warp::reply::json(&log_response))
        }
        Err(e) => {
            eprintln!("Error fetching pod logs: {}", e);
            let error_response = PodLogs {
                pod_name,
                namespace,
                logs: format!("Error fetching logs: {}", e),
            };
            Ok(warp::reply::json(&error_response))
        }
    }
}