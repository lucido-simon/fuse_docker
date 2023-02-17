use std::sync::Arc;

use super::docker_strategy::DockerError;
use fuser::FileAttr;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Copy)]
pub enum ParentDirectories {
    Root = 1,
    Containers = 2,
    Images = 3,
    Volumes = 4,
    Networks = 5,
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

// You can find additional implementation for each of the parent directories in other files in this module.
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

    pub(crate) fn attr(&self) -> FileAttr {
        match self {
            ParentDirectories::Containers => self.containers_attr(),
            ParentDirectories::Images => unimplemented!(),
            ParentDirectories::Volumes => unimplemented!(),
            ParentDirectories::Networks => unimplemented!(),
            ParentDirectories::Root => self.root_attr(),
        }
    }

    pub(crate) async fn read_dir(
        &self,
        offset: i64,
        reply: &mut fuser::ReplyDirectory,
        docker: Arc<Mutex<super::Docker>>,
    ) -> Result<(), libc::c_int> {
        match self {
            ParentDirectories::Containers => self.containers_read_dir(offset, reply, docker).await,
            ParentDirectories::Images => unimplemented!(),
            ParentDirectories::Volumes => unimplemented!(),
            ParentDirectories::Networks => unimplemented!(),
            ParentDirectories::Root => self.root_read_dir(offset, reply),
        }
    }

    pub(crate) async fn lookup(
        &self,
        name: &std::ffi::OsStr,
        reply: fuser::ReplyEntry,
        docker: Arc<Mutex<super::Docker>>,
    ) -> Result<(), libc::c_int> {
        match self {
            ParentDirectories::Containers => Self::containers_lookup(name, reply, docker).await,
            ParentDirectories::Images => unimplemented!(),
            ParentDirectories::Volumes => unimplemented!(),
            ParentDirectories::Networks => unimplemented!(),
            ParentDirectories::Root => Self::root_lookup(name, reply),
        }
    }

    pub(crate) fn ino_from_docker_name(name: &str) -> u64 {
        let mut ino = 0;
        for c in name.chars().take(8) {
            ino = ino << 8;
            ino += c as u64;
        }
        ino
    }

    pub(crate) fn docker_name_from_ino(ino: u64) -> String {
        let mut name = String::new();
        for i in 0..8 {
            let c = (ino >> (8 * (7 - i))) as u8 as char;
            if c != '\0' {
                name.push(c);
            }
        }
        name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ino_from_docker_name_and_back_short_name() {
        let name = "test";
        let ino = ParentDirectories::ino_from_docker_name(name);
        let name_back = ParentDirectories::docker_name_from_ino(ino);
        println!("{} {}", name, name_back);
        assert_eq!(name, name_back.as_str());
    }

    #[test]
    fn test_ino_from_docker_name_and_back_long_name() {
        let name = "this_is_a_very_long_name_but_the_conversion_is_still_working";
        let ino = ParentDirectories::ino_from_docker_name(name);
        let name_back = ParentDirectories::docker_name_from_ino(ino);
        println!("{} {}", name, name_back);
        assert_eq!(
            name.chars().take(8).collect::<String>().as_str(),
            name_back.as_str()
        );
    }
}
