use fuser::{FileAttr, FileType};
use std::{
    sync::Arc,
    time::{Duration, UNIX_EPOCH},
};
use tokio::sync::Mutex;

use crate::docker_strategy::{
    child_directories::child_directories::ChildDirectory, containers::Container,
    parent_directories::ParentDirectories,
};

impl ParentDirectories {
    pub(crate) async fn containers_root_lookup(
        name: &std::ffi::OsStr,
        reply: fuser::ReplyEntry,
        docker: Arc<Mutex<crate::docker_strategy::Docker>>,
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

        let containers: Vec<&Container> = docker
            .get::<Container>()
            .into_iter()
            .filter(|container| container.get_name().starts_with(container_name))
            .collect();

        if let Some(container) = containers.first() {
            reply.entry(&Duration::from_secs(1), &container.getattr(), 0);
            Ok(())
        } else {
            reply.error(libc::ENOENT);
            Ok(())
        }
    }

    pub(crate) fn containers_root_attr(&self) -> FileAttr {
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

    pub(crate) async fn containers_root_read_dir(
        &self,
        offset: i64,
        reply: &mut fuser::ReplyDirectory,
        docker: Arc<Mutex<crate::docker_strategy::Docker>>,
    ) -> Result<(), libc::c_int> {
        log::debug!("containers_read_dir(offset: {})", offset);

        let mut entries = vec![
            (ParentDirectories::Containers as u64, String::from(".")),
            (ParentDirectories::Root as u64, String::from("..")),
        ];

        let mut docker = docker.lock().await;

        if let Err(error) = docker.update_containers().await {
            log::debug!("Failed to update containers, error: {}", error);
        }

        let containers = docker.get::<Container>();

        log::debug!("containers_read_dir: containers: {:?}", containers);

        containers.into_iter().for_each(|container| {
            if let (Some(id), Some(names)) =
                (container.container.id.as_ref(), &container.container.names)
            {
                let name_string = names.first().unwrap().to_owned();

                entries.push((Self::ino_from_docker_id(id), name_string))
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
}
