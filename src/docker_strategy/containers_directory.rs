use std::time::UNIX_EPOCH;

use fuser::{FileAttr, FileType};

use crate::docker_strategy::parent_directories::ParentDirectories;

impl ParentDirectories {
    pub(crate) fn containers_lookup(
        _name: &std::ffi::OsStr,
        reply: fuser::ReplyEntry,
    ) -> Result<(), libc::c_int> {
        reply.error(libc::ENOENT);
        Ok(())
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

    pub(crate) fn containers_read_dir(
        &self,
        offset: i64,
        reply: &mut fuser::ReplyDirectory,
    ) -> Result<(), libc::c_int> {
        log::debug!("containers_read_dir(offset: {})", offset);

        let entries = [
            (ParentDirectories::Containers as u64, "."),
            (ParentDirectories::Root as u64, ".."),
        ];

        for (i, (ino, name)) in entries.iter().enumerate().skip(offset as usize) {
            if i as i64 >= offset {
                if reply.add(*ino, i as i64 + 1, FileType::Directory, name) {
                    break;
                }
            }
        }

        Ok(())
    }
}
