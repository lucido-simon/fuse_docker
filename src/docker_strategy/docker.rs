use std::{collections::HashMap, time::Duration};

use bollard::container::ListContainersOptions;
use tokio::{runtime::Handle, time::Instant};

use crate::docker_strategy::containers::Container;

use super::child_directories::child_directories::ChildDirectory;

const TTL: Duration = Duration::from_secs(5);

pub struct Docker {
    docker: bollard::Docker,
    mappings: HashMap<u64, Box<dyn ChildDirectory>>,
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

    pub fn get_child(&self, inode: u64) -> Option<&Box<dyn ChildDirectory>> {
        self.mappings.get(&inode)

        // .and_then(|child| {
        //     if let Some(child) = child.downcast_ref::<Box<dyn ChildDirectory>>() {
        //         log::debug!("Found child: {:?}", child);
        //         Some(child)
        //     } else {
        //         log::debug!("No child: {:?}, {:?}", child, self.mappings);
        //         None
        //     }
        // })
    }

    async fn force_update_containers(&mut self) -> Result<(), bollard::errors::Error> {
        self.docker
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
            .for_each(|container_summary| {
                let container = Container::new(container_summary);
                let inode = container.get_ino();
                self.mappings.insert(inode, Box::new(container));
            });

        self.clock_since_last_update = Instant::now();
        Ok(())
    }

    pub async fn update_containers(&mut self) -> Result<(), bollard::errors::Error> {
        if self.clock_since_last_update.elapsed() < TTL {
            return Ok(());
        }

        self.force_update_containers().await
    }

    pub fn get<T>(&self) -> Vec<&T>
    where
        T: ChildDirectory,
    {
        self.mappings
            .values()
            .filter_map(|child| child.as_any().downcast_ref::<T>())
            .collect()
    }

    pub fn get_docker(&self) -> &bollard::Docker {
        &self.docker
    }
}
