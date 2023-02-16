use std::ffi::OsStr;

use fuser::Request;

pub trait FileSystemStrategy {
    fn init(&self) -> Result<(), libc::c_int>;
    fn lookup(
        &self,
        req: &Request<'_>,
        parent: u64,
        name: &OsStr,
        reply: fuser::ReplyEntry,
    ) -> Result<(), libc::c_int>;
    fn open(
        &self,
        req: &Request<'_>,
        ino: u64,
        flags: i32,
        reply: fuser::ReplyOpen,
    ) -> Result<(), libc::c_int>;
    fn getattr(
        &self,
        req: &Request<'_>,
        ino: u64,
        reply: fuser::ReplyAttr,
    ) -> Result<(), libc::c_int>;
    fn readdir(
        &self,
        req: &Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        reply: &mut fuser::ReplyDirectory,
    ) -> Result<(), libc::c_int>;
}
