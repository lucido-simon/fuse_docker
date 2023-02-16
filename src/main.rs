mod docker_strategy;
mod fuse_handler;

use docker_strategy::DockerStrategy;
use fuse_handler::FuseHandler;
use fuser::MountOption;

fn main() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));
    let mountpoint = "/tmp/fuse";

    let strategy = DockerStrategy::new();
    let handler = FuseHandler::new(Box::new(strategy));

    fuser::mount2(
        handler,
        mountpoint,
        &[
            MountOption::RW,
            MountOption::FSName("docker_fuse".to_string()),
        ],
    )
    .unwrap();
}
