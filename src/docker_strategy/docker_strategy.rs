use std::sync::Arc;
use std::time::Duration;

use bimap::BiMap;
use fuser::Request;
use tokio::runtime::Handle;
use tokio::sync::Mutex;

use crate::docker_strategy::parent_directories::ParentDirectories;
use crate::fuse_handler::FileSystemStrategy;

pub struct DockerStrategy {
    _bimap: BiMap<u64, String>,
    docker: Arc<Mutex<super::Docker>>,
}

pub enum DockerError {
    UnknownParentDirectory,
}

impl DockerStrategy {
    pub fn new() -> Self {
        let mut bimap = BiMap::new();
        ParentDirectories::iterator().for_each(|x| {
            bimap.insert(x as u64, x.to_string());
        });

        let docker = super::Docker::new();

        log::info!(target: "Docker", "DockerStrategy initialized");

        Self {
            _bimap: bimap,
            docker: Arc::new(Mutex::new(docker)),
        }
    }
}

impl FileSystemStrategy for DockerStrategy {
    fn init(&self) -> Result<(), libc::c_int> {
        let docker = self.docker.blocking_lock();
        Handle::current()
            .block_on(docker.get_docker().ping())
            .map_err(|e| {
                log::error!("Failed to ping docker daemon: {}", e);
                log::error!("Is the docker daemon running ?");
                log::error!("Are you running as root ?");
                libc::EACCES
            })?;

        log::info!(target: "Docker", "DockerStrategy initialized");

        Ok(())
    }

    fn lookup(
        &self,
        _req: &fuser::Request<'_>,
        parent: u64,
        name: &std::ffi::OsStr,
        reply: fuser::ReplyEntry,
    ) -> Result<(), libc::c_int> {
        if let Ok(parent) = ParentDirectories::try_from(parent) {
            log::debug!("lookup: parent: {:?}, name: {:?}", parent, name);
            Handle::current().block_on(parent.lookup(name, reply, self.docker.clone()))
        } else {
            reply.error(libc::ENOENT);
            Ok(())
        }
    }

    fn open(
        &self,
        _req: &fuser::Request<'_>,
        _ino: u64,
        _flags: i32,
        reply: fuser::ReplyOpen,
    ) -> Result<(), libc::c_int> {
        reply.opened(0, 0);
        Ok(())
    }

    fn getattr(
        &self,
        _req: &Request<'_>,
        ino: u64,
        reply: fuser::ReplyAttr,
    ) -> Result<(), libc::c_int> {
        if let Ok(parent) = ParentDirectories::try_from(ino) {
            reply.attr(&Duration::from_secs(1), &parent.attr());
            Ok(())
        } else {
            reply.error(libc::ENOENT);
            Ok(())
        }
    }

    fn readdir(
        &self,
        _req: &Request<'_>,
        ino: u64,
        _fh: u64,
        offset: i64,
        reply: &mut fuser::ReplyDirectory,
    ) -> Result<(), libc::c_int> {
        if let Ok(parent) = ParentDirectories::try_from(ino) {
            Handle::current().block_on(parent.read_dir(offset, reply, self.docker.clone()))
        } else {
            Err(libc::ENOENT)
        }
    }
}
