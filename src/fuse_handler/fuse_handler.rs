use fuser::Filesystem;

use super::FileSystemStrategy;

pub struct FuseHandler {
    handler: Box<dyn FileSystemStrategy>,
}

impl FuseHandler {
    pub fn new(handler: Box<dyn FileSystemStrategy>) -> Self {
        Self { handler }
    }
}

impl Filesystem for FuseHandler {
    fn init(
        &mut self,
        _req: &fuser::Request<'_>,
        _config: &mut fuser::KernelConfig,
    ) -> Result<(), libc::c_int> {
        self.handler.init()?;
        Ok(())
    }
}
