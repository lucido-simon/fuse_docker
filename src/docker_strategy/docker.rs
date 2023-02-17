use std::time::Duration;

use bollard::{container::ListContainersOptions, service::ContainerSummary};
use tokio::time::Instant;

const TTL: Duration = Duration::from_secs(5);

pub(crate) struct Docker {
    docker: bollard::Docker,
    containers: Vec<ContainerSummary>,
    clock_since_last_update: Instant,
}

impl Docker {
    pub fn new() -> Self {
        let docker = match bollard::Docker::connect_with_local_defaults() {
            Ok(docker) => docker,
            Err(e) => {
                log::error!("Failed to connect to docker daemon: {}", e);
                panic!("Failed to connect to docker daemon: {}", e)
            }
        };

        Self {
            docker,
            containers: Vec::<ContainerSummary>::new(),
            clock_since_last_update: Instant::now(),
        }
    }

    pub async fn update_containers(&mut self) -> Result<(), bollard::errors::Error> {
        if self.clock_since_last_update.elapsed() < TTL {
            return Ok(());
        }
        self.containers = self
            .docker
            .list_containers(Some(ListContainersOptions::<String> {
                all: true,
                ..Default::default()
            }))
            .await?
            .into_iter()
            .map(|container| {
                let names = container
                    .names
                    .unwrap_or_default()
                    .into_iter()
                    .map(|name| name.trim_start_matches('/').to_string())
                    .collect();
                ContainerSummary {
                    names: Some(names),
                    ..container
                }
            })
            .collect();

        self.clock_since_last_update = Instant::now();
        Ok(())
    }

    pub fn get_containers(&self) -> &Vec<ContainerSummary> {
        &self.containers
    }

    pub fn get_docker(&self) -> &bollard::Docker {
        &self.docker
    }
}
