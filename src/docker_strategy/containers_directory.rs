use bollard::service::ContainerSummary;
use fuser::{FileAttr, FileType};
use std::{
    sync::Arc,
    time::{Duration, UNIX_EPOCH},
};
use tokio::sync::Mutex;

use crate::docker_strategy::parent_directories::ParentDirectories;

impl ParentDirectories {
    pub(crate) async fn containers_lookup(
        name: &std::ffi::OsStr,
        reply: fuser::ReplyEntry,
        docker: Arc<Mutex<super::Docker>>,
    ) -> Result<(), libc::c_int> {
        let mut docker = docker.lock().await;
        if let Err(e) = docker.update_containers().await {
            log::error!("Failed to update containers: {}", e);
        }

        let container_name = if let Some(name) = name.to_str() {
            name
        } else {
            reply.error(libc::ENOENT);
            return Ok(());
        };

        let containers: Vec<&ContainerSummary> = docker
            .get_containers()
            .into_iter()
            .filter(|container| {
                if let Some(names) = &container.names {
                    names.iter().any(|name| container_name.starts_with(name))
                } else {
                    false
                }
            })
            .collect();

        if let Some(container) = containers.first() {
            if let Some(id) = container.id.as_ref() {
                reply.entry(
                    &Duration::from_secs(1),
                    &Self::create_container_directory(container, id),
                    0,
                );
                Ok(())
            } else {
                reply.error(libc::ENOENT);
                Ok(())
            }
        } else {
            reply.error(libc::ENOENT);
            Ok(())
        }
    }

    pub(crate) fn containers_attr(&self) -> FileAttr {
        FileAttr {
            ino: self.into(),
            size: 0,
            blocks: 0,
            atime: UNIX_EPOCH, // 1970-01-01 00:00:00
            mtime: UNIX_EPOCH,
            ctime: UNIX_EPOCH,
            crtime: UNIX_EPOCH,
            kind: FileType::Directory,
            perm: 0o777,
            nlink: 1,
            uid: 0,
            gid: 0,
            rdev: 0,
            flags: 0,
            blksize: 0,
        }
    }

    pub(crate) async fn containers_read_dir(
        &self,
        offset: i64,
        reply: &mut fuser::ReplyDirectory,
        docker: Arc<Mutex<super::Docker>>,
    ) -> Result<(), libc::c_int> {
        log::debug!("containers_read_dir(offset: {})", offset);

        let mut entries = vec![
            (ParentDirectories::Containers as u64, "."),
            (ParentDirectories::Root as u64, ".."),
        ];

        let mut docker = docker.lock().await;

        if let Err(error) = docker.update_containers().await {
            log::debug!("Failed to update containers, error: {}", error);
        }

        let containers = docker.get_containers();

        containers.iter().for_each(|container| {
            if let (Some(id), Some(names)) = (container.id.as_ref(), container.names.as_ref()) {
                let name_string = names.first().unwrap();

                entries.push((Self::ino_from_docker_name(id.as_str()), name_string))
            }
        });

        for (i, (ino, name)) in entries.iter().enumerate().skip(offset as usize) {
            log::debug!("containers_read_dir: {} {} {}", i, ino, name);
            if reply.add(*ino, i as i64 + 1, FileType::Directory, name) {
                break;
            }
        }

        Ok(())
    }

    fn create_container_directory(container: &ContainerSummary, id: &String) -> FileAttr {
        FileAttr {
            ino: Self::ino_from_docker_name(id.as_str()),
            size: 0,
            blocks: 0,
            atime: UNIX_EPOCH, // 1970-01-01 00:00:00
            mtime: UNIX_EPOCH,
            ctime: UNIX_EPOCH,
            crtime: UNIX_EPOCH,
            kind: FileType::Directory,
            perm: 0o777,
            nlink: 1,
            uid: 0,
            gid: 0,
            rdev: 0,
            flags: 0,
            blksize: 0,
        }
    }
}
