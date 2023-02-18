use bollard::service::ContainerSummary;

use crate::docker_strategy::parent_directories::ParentDirectories;

#[derive(Debug)]
pub struct Container {
    pub ino: u64,
    pub container: ContainerSummary,
}

impl<'a> FromIterator<&'a Container> for Vec<&'a ContainerSummary> {
    fn from_iter<T: IntoIterator<Item = &'a Container>>(iter: T) -> Self {
        iter.into_iter()
            .map(|container| &container.container)
            .collect()
    }
}

impl ParentDirectories {}
