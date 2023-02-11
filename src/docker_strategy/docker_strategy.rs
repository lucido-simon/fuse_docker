use crate::fuse_handler::FileSystemStrategy;

pub struct DockerStrategy {}

impl DockerStrategy {
    pub fn new() -> Self {
        Self {}
    }
}

impl FileSystemStrategy for DockerStrategy {
    fn init(&self) -> Result<(), libc::c_int> {
        Ok(())
    }
}
