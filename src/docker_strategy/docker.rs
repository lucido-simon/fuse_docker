use std::{collections::HashMap, time::Duration};

use bollard::{container::ListContainersOptions, service::ContainerSummary};
use tokio::{runtime::Handle, time::Instant};

use crate::docker_strategy::{containers::Container, parent_directories::ParentDirectories};

const TTL: Duration = Duration::from_secs(5);

pub(crate) struct Docker {
    docker: bollard::Docker,
    mappings: HashMap<u64, ParentDirectories>,
    containers: Vec<Container>,
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

        let mut docker = Self {
            docker,
            containers: Vec::<Container>::new(),
            clock_since_last_update: Instant::now(),
            mappings: HashMap::new(),
        };

        Handle::current().block_on(async {
            if let Err(e) = docker.force_update_containers().await {
                log::error!("Failed to update containers: {}", e);
            }
        });

        docker
    }

    async fn force_update_containers(&mut self) -> Result<(), bollard::errors::Error> {
        self.containers = self
            .docker
            .list_containers(Some(ListContainersOptions::<String> {
                all: true,
                ..Default::default()
            }))
            .await?
            .into_iter()
            .filter(|container| {
                if let (Some(names), Some(_)) = (&container.names, &container.id) {
                    names.iter().any(|name| name.starts_with("/"))
                } else {
                    false
                }
            })
            .map(|container| {
                let names = container
                    .names
                    .unwrap_or_default()
                    .into_iter()
                    .map(|name| name.trim_start_matches('/').to_string())
                    .collect();

                let inode = if let Some(id) = container.id.as_ref() {
                    let inode = ParentDirectories::ino_from_docker_id(id);
                    self.mappings.insert(inode, ParentDirectories::Containers);
                    inode
                } else {
                    panic!("Container without id")
                };
                Container {
                    ino: inode,
                    container: ContainerSummary {
                        names: Some(names),
                        ..container
                    },
                }
            })
            .collect();

        self.clock_since_last_update = Instant::now();
        Ok(())
    }

    pub async fn update_containers(&mut self) -> Result<(), bollard::errors::Error> {
        if self.clock_since_last_update.elapsed() < TTL {
            return Ok(());
        }

        self.force_update_containers().await
    }

    pub fn get_containers(&self) -> &Vec<Container> {
        &self.containers
    }

    pub fn get_docker(&self) -> &bollard::Docker {
        &self.docker
    }
}
