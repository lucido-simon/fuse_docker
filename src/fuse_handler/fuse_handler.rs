use fuser::{Filesystem, Request};

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
        log::debug!("init");
        self.handler.init()?;
        Ok(())
    }

    fn lookup(
        &mut self,
        _req: &fuser::Request<'_>,
        parent: u64,
        name: &std::ffi::OsStr,
        reply: fuser::ReplyEntry,
    ) {
        log::debug!(
            "lookup on parent {} with name {}",
            parent,
            name.to_str().unwrap()
        );
        self.handler.lookup(_req, parent, name, reply).unwrap();
    }

    fn open(&mut self, _req: &Request<'_>, _ino: u64, _flags: i32, reply: fuser::ReplyOpen) {
        log::debug!("open on ino {} ", _ino);
        self.handler.open(_req, _ino, _flags, reply).unwrap();
    }

    fn getattr(&mut self, _req: &Request<'_>, ino: u64, reply: fuser::ReplyAttr) {
        log::debug!("getattr on ino {} ", ino);
        self.handler.getattr(_req, ino, reply).unwrap();
    }

    fn readdir(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        mut reply: fuser::ReplyDirectory,
    ) {
        log::debug!(
            "readdir on ino {} with fh {} and offset {}",
            ino,
            fh,
            offset
        );

        match self.handler.readdir(_req, ino, fh, offset, &mut reply) {
            Ok(_) => reply.ok(),
            Err(e) => reply.error(e),
        }
    }
}
