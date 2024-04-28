use std::error::Error;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use bollard::models::ContainerSummary;
use bollard::container::StatsOptions;
use log::{info, error, debug};

extern crate pretty_env_logger;


use bollard::Docker;
use bollard::container::ListContainersOptions;
// use tokio::task;
use tokio::signal;
use tokio_util::task::TaskTracker;
use tokio_util::sync::CancellationToken;
use tokio_stream::StreamExt;

type ContainerStats = Arc<Mutex<HashMap<String, ContainerSummary>>>;

// struct ContainerStats {
//     container_stats: TContainerStats,
// }


#[cfg(unix)]
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();

    info!("Assistant Ref Taking Position");
    let tracker = TaskTracker::new();
    let token = CancellationToken::new();

    let container_stats = Arc::new(Mutex::new(HashMap::new()));

    let cloned_token = token.clone();
    let container_stats_handle = container_stats.clone();
    tracker.spawn(async move {
        tokio::select! {
            _ = cloned_token.cancelled() => {
                info!("Shutting down get_docker_stats");
            }
            _ = get_docker_stats(10, container_stats_handle) => {
                // unreachable due to loop
            }
        }
    });

    let cloned_token = token.clone();
    let container_stats_handle = container_stats.clone();
    tracker.spawn(async move {
        tokio::select! {
            _ = cloned_token.cancelled() => {
                info!("Shutting down send_docker_stats");
            }
            _ = send_docker_stats(10, container_stats_handle) => {
                // unreachable due to loop
            }
        }
    });

    match signal::ctrl_c().await {
        Ok(()) => {},
        Err(err) => {
            error!("Unable to listen for shutdown signal: {}", err);
            // we also shut down in case of error
        },
    }
    info!("Shutting down");
    token.cancel();
    tracker.close();
    tracker.wait().await;

    Ok(())
}

async fn get_docker_stats(sleep: u64, container_stats: ContainerStats) {
    loop {
        // In order to maintain the Send trait, this has to be scoped
        {
            info!("Connecting to the docker engine socket");
            let docker = Docker::connect_with_socket_defaults().unwrap();
            
            let version = docker.version().await.unwrap();
            info!("{}", version.platform.unwrap().name);

            // let mut filters = HashMap::new();
            // filters.insert("health", vec!["healthy"]);

            let options = Some(ListContainersOptions::<String>{
                all: true,
                // filters,
                ..Default::default()
            });
            let containers = docker.list_containers(options).await.unwrap();
            debug!("{:?}", containers);

            let mut current_stats = container_stats.lock().unwrap();
            for container in containers {
                let id = container.id.clone().unwrap();
                current_stats.insert(id, container.clone());
            }

            // let options = Some(StatsOptions{
            //     stream: false,
            //     one_shot: true,
            // });

            // for (id, _) in current_stats {
            //     let mut stats = docker.stats(id, options);
            //     while let Some(stat) = stats.next().await {
            //         debug!("{:?}", stat);
            //     }
            // }
        }
        tokio::time::sleep(std::time::Duration::from_secs(sleep)).await;
    }
}

async fn send_docker_stats(sleep: u64, current_stats: ContainerStats) {
    loop {
        {
            let current_stats = current_stats.lock().unwrap();
            // debug!("Current Stats: {:?}", current_stats);
            for (id, container) in current_stats.iter() {
                // debug!("Container: {} - {:?}", id, container);
                info!("Container: {} - {} - {}", &id[0..12], container.names.as_ref().unwrap().join(", "), container.status.as_ref().unwrap());
            }
        }
        tokio::time::sleep(std::time::Duration::from_secs(sleep)).await;
    }
}
