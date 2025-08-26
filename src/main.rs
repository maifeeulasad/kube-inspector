// src/main.rs
use anyhow::Result;
use std::convert::Infallible;
use warp::Filter;

mod handlers;
mod models;
mod k8s_client;

use handlers::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // Initialize Kubernetes client
    let client = kube::Client::try_default().await?;
    
    // CORS configuration
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type"])
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE"]);

    // API routes
    let api_routes = warp::path("api")
        .and(
            warp::path("namespaces")
                .and(warp::get())
                .and(with_client(client.clone()))
                .and_then(get_namespaces)
                .or(warp::path("pods")
                    .and(warp::path::param::<String>())
                    .and(warp::get())
                    .and(with_client(client.clone()))
                    .and_then(get_pods))
                .or(warp::path("services")
                    .and(warp::path::param::<String>())
                    .and(warp::get())
                    .and(with_client(client.clone()))
                    .and_then(get_services))
                .or(warp::path("deployments")
                    .and(warp::path::param::<String>())
                    .and(warp::get())
                    .and(with_client(client.clone()))
                    .and_then(get_deployments))
                .or(warp::path("configmaps")
                    .and(warp::path::param::<String>())
                    .and(warp::get())
                    .and(with_client(client.clone()))
                    .and_then(get_configmaps))
                .or(warp::path("networkpolicies")
                    .and(warp::path::param::<String>())
                    .and(warp::get())
                    .and(with_client(client.clone()))
                    .and_then(get_network_policies))
                .or(warp::path("pod")
                    .and(warp::path::param::<String>())
                    .and(warp::path::param::<String>())
                    .and(warp::get())
                    .and(with_client(client.clone()))
                    .and_then(get_pod_details))
                .or(warp::path("pod")
                    .and(warp::path::param::<String>())
                    .and(warp::path::param::<String>())
                    .and(warp::path("logs"))
                    .and(warp::get())
                    .and(with_client(client.clone()))
                    .and_then(get_pod_logs))
        );

    // Static file serving
    let static_files = warp::fs::dir("static");

    // Root redirect
    let root = warp::path::end().map(|| {
        warp::redirect::temporary(warp::http::Uri::from_static("/index.html"))
    });

    let routes = root
        .or(api_routes)
        .or(static_files)
        .with(warp::log("k8s_dashboard"))
        .with(cors);

    println!("🚀 Kubernetes Dashboard starting on http://0.0.0.0:8080");
    
    warp::serve(routes)
        .run(([0, 0, 0, 0], 8080))
        .await;

    Ok(())
}

fn with_client(client: kube::Client) -> impl Filter<Extract = (kube::Client,), Error = Infallible> + Clone {
    warp::any().map(move || client.clone())
}