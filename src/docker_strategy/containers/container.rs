use std::time::{Duration, UNIX_EPOCH};

use bollard::service::ContainerSummary;
use fuser::{FileAttr, FileType};

use crate::docker_strategy::{
    child_directories::child_directories::ChildDirectory, parent_directories::ParentDirectories,
};

#[derive(Debug)]
pub struct Container {
    ino: u64,
    name: String,
    pub container: ContainerSummary,
}

impl From<Container> for Box<dyn ChildDirectory> {
    fn from(value: Container) -> Self {
        Box::new(value)
    }
}

impl ChildDirectory for Container {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_ino(&self) -> u64 {
        self.ino
    }

    fn get_id(&self) -> &String {
        self.container.id.as_ref().unwrap()
    }

    fn get_name(&self) -> &String {
        &self.name
    }

    fn get_parent(&self) -> ParentDirectories {
        ParentDirectories::Containers
    }

    fn read_dir(&self, offset: i64, reply: &mut fuser::ReplyDirectory) -> Result<(), libc::c_int> {
        log::debug!("containers_read_dir(offset: {})", offset);

        let entries = vec![
            (self.ino, String::from(".")),
            (ParentDirectories::Containers as u64, String::from("..")),
        ];

        for (i, (ino, name)) in entries.iter().enumerate().skip(offset as usize) {
            log::debug!("containers_read_dir: {} {} {}", i, ino, name);
            if reply.add(*ino, i as i64 + 1, FileType::Directory, name) {
                break;
            }
        }

        Ok(())
    }

    fn lookup(&self, name: &str) -> Result<(), libc::c_int> {
        todo!()
    }

    fn getattr(&self) -> FileAttr {
        let time =
            UNIX_EPOCH + Duration::from_secs(self.container.created.unwrap_or_default() as u64);

        FileAttr {
            ino: self.ino,
            size: 0,
            blocks: 0,
            atime: UNIX_EPOCH,
            mtime: time,
            ctime: time,
            crtime: time,
            kind: FileType::Directory,
            perm: 0o755,
            nlink: 0,
            uid: 0,
            gid: 0,
            rdev: 0,
            flags: 0,
            blksize: 0,
        }
    }
}

impl Container {
    pub fn new(container: ContainerSummary) -> Self {
        let names: Vec<String> = container
            .names
            .unwrap_or_default()
            .into_iter()
            .map(|name| name.trim_start_matches('/').to_string())
            .collect();

        let ino = if let Some(id) = container.id.as_ref() {
            ParentDirectories::ino_from_docker_id(id)
        } else {
            panic!("Container without id")
        };

        Self {
            container: ContainerSummary {
                names: Some(names.clone()),
                ..container
            },
            name: names.first().unwrap().to_string(),
            ino,
        }
    }
}

impl FromIterator<Container> for Vec<ContainerSummary> {
    fn from_iter<T: IntoIterator<Item = Container>>(iter: T) -> Self {
        iter.into_iter()
            .map(|container| container.container)
            .collect()
    }
}

impl<'a> FromIterator<&'a Container> for Vec<&'a ContainerSummary> {
    fn from_iter<T: IntoIterator<Item = &'a Container>>(iter: T) -> Self {
        iter.into_iter()
            .map(|container| &container.container)
            .collect()
    }
}
