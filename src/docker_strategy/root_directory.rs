use std::time::{Duration, UNIX_EPOCH};

use fuser::{FileAttr, FileType};

use crate::docker_strategy::parent_directories::ParentDirectories;

impl ParentDirectories {
    pub(crate) fn root_lookup(
        name: &std::ffi::OsStr,
        reply: fuser::ReplyEntry,
    ) -> Result<(), libc::c_int> {
        // TODO: Check if there is RFC 3137 in nightly for let else or a better workaround ?

        let name = match name.to_str() {
            Some(str) => str,
            _ => {
                reply.error(libc::ENOENT);
                return Ok(());
            }
        };

        match name.try_into() {
            Ok(ParentDirectories::Containers) => {
                reply.entry(
                    &Duration::from_secs(1),
                    &ParentDirectories::Containers.attr(),
                    0,
                );
                Ok(())
            }

            _ => {
                reply.error(libc::ENOENT);
                Ok(())
            }
        }
    }

    pub(crate) fn root_attr(&self) -> FileAttr {
        FileAttr {
            ino: self.into(),
            size: Self::iterator().count() as u64,
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

    pub(crate) fn root_read_dir(
        &self,
        offset: i64,
        reply: &mut fuser::ReplyDirectory,
    ) -> Result<(), libc::c_int> {
        let entries = [
            (ParentDirectories::Root as u64, ".".to_string()),
            (ParentDirectories::Root as u64, "..".to_string()),
            (
                ParentDirectories::Containers as u64,
                ParentDirectories::Containers.to_string(),
            ),
            (
                ParentDirectories::Images as u64,
                ParentDirectories::Images.to_string(),
            ),
            (
                ParentDirectories::Volumes as u64,
                ParentDirectories::Volumes.to_string(),
            ),
            (
                ParentDirectories::Networks as u64,
                ParentDirectories::Networks.to_string(),
            ),
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
