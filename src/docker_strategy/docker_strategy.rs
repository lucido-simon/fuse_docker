use std::time::{Duration, UNIX_EPOCH};

use bimap::BiMap;
use fuser::{FileAttr, FileType, Request};

use crate::fuse_handler::FileSystemStrategy;

pub struct DockerStrategy {
    bimap: BiMap<u64, String>,
}

#[derive(Debug, Clone, Copy)]
pub enum ParentDirectories {
    Root = 1,
    Containers = 2,
    Images = 3,
    Volumes = 4,
    Networks = 5,
}

pub enum DockerError {
    UnknownParentDirectory,
}

impl From<ParentDirectories> for u64 {
    fn from(value: ParentDirectories) -> Self {
        match value {
            ParentDirectories::Root => 1,
            ParentDirectories::Containers => 2,
            ParentDirectories::Images => 3,
            ParentDirectories::Volumes => 4,
            ParentDirectories::Networks => 5,
        }
    }
}

impl From<&ParentDirectories> for u64 {
    fn from(value: &ParentDirectories) -> Self {
        match value {
            ParentDirectories::Root => 1,
            ParentDirectories::Containers => 2,
            ParentDirectories::Images => 3,
            ParentDirectories::Volumes => 4,
            ParentDirectories::Networks => 5,
        }
    }
}

impl TryFrom<u64> for ParentDirectories {
    type Error = DockerError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(ParentDirectories::Root),
            2 => Ok(ParentDirectories::Containers),
            3 => Ok(ParentDirectories::Images),
            4 => Ok(ParentDirectories::Volumes),
            5 => Ok(ParentDirectories::Networks),
            _ => Err(DockerError::UnknownParentDirectory),
        }
    }
}

impl TryFrom<&str> for ParentDirectories {
    type Error = DockerError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "root" => Ok(ParentDirectories::Root),
            "containers" => Ok(ParentDirectories::Containers),
            "images" => Ok(ParentDirectories::Images),
            "volumes" => Ok(ParentDirectories::Volumes),
            "networks" => Ok(ParentDirectories::Networks),
            _ => Err(DockerError::UnknownParentDirectory),
        }
    }
}

impl ParentDirectories {
    pub fn iterator() -> impl Iterator<Item = ParentDirectories> {
        [
            ParentDirectories::Root,
            ParentDirectories::Containers,
            ParentDirectories::Images,
            ParentDirectories::Volumes,
            ParentDirectories::Networks,
        ]
        .iter()
        .copied()
    }

    pub fn to_string(&self) -> String {
        match self {
            ParentDirectories::Root => String::from("/"),
            ParentDirectories::Containers => String::from("containers"),
            ParentDirectories::Images => String::from("images"),
            ParentDirectories::Volumes => String::from("volumes"),
            ParentDirectories::Networks => String::from("networks"),
        }
    }

    pub fn attr(&self) -> FileAttr {
        match self {
            ParentDirectories::Containers => unimplemented!(),
            ParentDirectories::Images => unimplemented!(),
            ParentDirectories::Volumes => unimplemented!(),
            ParentDirectories::Networks => unimplemented!(),
            ParentDirectories::Root => self.root_attr(),
        }
    }

    pub fn read_dir(
        &self,
        offset: i64,
        reply: &mut fuser::ReplyDirectory,
    ) -> Result<(), libc::c_int> {
        match self {
            ParentDirectories::Containers => unimplemented!(),
            ParentDirectories::Images => unimplemented!(),
            ParentDirectories::Volumes => unimplemented!(),
            ParentDirectories::Networks => unimplemented!(),
            ParentDirectories::Root => Self::root_read_dir(offset, reply),
        }
    }

    pub fn lookup(
        &self,
        name: &std::ffi::OsStr,
        reply: fuser::ReplyEntry,
    ) -> Result<(), libc::c_int> {
        match self {
            ParentDirectories::Containers => unimplemented!(),
            ParentDirectories::Images => unimplemented!(),
            ParentDirectories::Volumes => unimplemented!(),
            ParentDirectories::Networks => unimplemented!(),
            ParentDirectories::Root => self.root_lookup(name, reply),
        }
    }

    fn root_lookup(
        &self,
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
                    &FileAttr {
                        ino: ParentDirectories::Containers as u64,
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
                    },
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

    fn root_attr(&self) -> FileAttr {
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

    fn root_read_dir(offset: i64, reply: &mut fuser::ReplyDirectory) -> Result<(), libc::c_int> {
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

impl DockerStrategy {
    pub fn new() -> Self {
        let mut bimap = BiMap::new();
        ParentDirectories::iterator().for_each(|x| {
            bimap.insert(x as u64, x.to_string());
        });

        Self { bimap }
    }
}

impl FileSystemStrategy for DockerStrategy {
    fn init(&self) -> Result<(), libc::c_int> {
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
            return parent.lookup(name, reply);
        }
        Err(libc::ENOENT)
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
            return Ok(());
        }
        Err(libc::ENOENT)
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
            parent.read_dir(offset, reply)
        } else {
            Err(libc::ENOENT)
        }
    }
}
