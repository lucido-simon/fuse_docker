mod docker_strategy;
mod fuse_handler;

use docker_strategy::DockerStrategy;
use fuse_handler::FuseHandler;
use fuser::MountOption;

fn main() {
    let mountpoint = "/tmp/fuse";

    let strategy = DockerStrategy::new();
    let handler = FuseHandler::new(Box::new(strategy));

    fuser::mount2(handler, mountpoint, &[MountOption::RW]).unwrap();
}
