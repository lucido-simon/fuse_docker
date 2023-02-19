use std::{any::Any, fmt::Debug};

use crate::docker_strategy::{
    containers::container::Container, parent_directories::ParentDirectories,
};

use fuser::FileAttr;

pub trait ChildDirectory: Debug + Any + 'static {
    fn as_any(&self) -> &dyn Any;
    fn get_ino(&self) -> u64;
    fn get_id(&self) -> &String;
    fn get_name(&self) -> &String;
    fn get_parent(&self) -> ParentDirectories;

    fn try_into_container(self: Box<Self>) -> Option<Container> {
        match self.get_parent() {
            _ => None,
        }
    }

    fn read_dir(&self, offset: i64, reply: &mut fuser::ReplyDirectory) -> Result<(), libc::c_int>;
    fn lookup(&self, name: &str) -> Result<(), libc::c_int>;
    fn getattr(&self) -> FileAttr;
}

// pub enum ChildDirectories {
//     Containers,
//     Images,
//     Volumes,
//     Networks,
// }

// impl From<ChildDirectories> for u64 {
//     fn from(value: ChildDirectories) -> Self {
//         match value {
//             ChildDirectories::Root => 1,
//             ChildDirectories::Containers => 2,
//             ChildDirectories::Images => 3,
//             ChildDirectories::Volumes => 4,
//             ChildDirectories::Networks => 5,
//         }
//     }
// }

// impl From<&ChildDirectories> for u64 {
//     fn from(value: &ChildDirectories) -> Self {
//         match value {
//             ChildDirectories::Root => 1,
//             ChildDirectories::Containers => 2,
//             ChildDirectories::Images => 3,
//             ChildDirectories::Volumes => 4,
//             ChildDirectories::Networks => 5,
//         }
//     }
// }

// impl TryFrom<u64> for ChildDirectories {
//     type Error = DockerError;

//     fn try_from(value: u64) -> Result<Self, Self::Error> {
//         match value {
//             1 => Ok(ChildDirectories::Root),
//             2 => Ok(ChildDirectories::Containers),
//             3 => Ok(ChildDirectories::Images),
//             4 => Ok(ChildDirectories::Volumes),
//             5 => Ok(ChildDirectories::Networks),
//             _ => Err(DockerError::UnknownParentDirectory),
//         }
//     }
// }

// impl TryFrom<&str> for ChildDirectories {
//     type Error = DockerError;

//     fn try_from(value: &str) -> Result<Self, Self::Error> {
//         match value {
//             "root" => Ok(ChildDirectories::Root),
//             "containers" => Ok(ChildDirectories::Containers),
//             "images" => Ok(ChildDirectories::Images),
//             "volumes" => Ok(ChildDirectories::Volumes),
//             "networks" => Ok(ChildDirectories::Networks),
//             _ => Err(DockerError::UnknownParentDirectory),
//         }
//     }
// }

// // You can find additional implementation for each of the parent directories in other files in this module.
// impl ChildDirectories {
//     pub fn iterator() -> impl Iterator<Item = ChildDirectories> {
//         [
//             ChildDirectories::Root,
//             ChildDirectories::Containers,
//             ChildDirectories::Images,
//             ChildDirectories::Volumes,
//             ChildDirectories::Networks,
//         ]
//         .iter()
//         .copied()
//     }

//     pub fn to_string(&self) -> String {
//         match self {
//             ChildDirectories::Root => String::from("/"),
//             ChildDirectories::Containers => String::from("containers"),
//             ChildDirectories::Images => String::from("images"),
//             ChildDirectories::Volumes => String::from("volumes"),
//             ChildDirectories::Networks => String::from("networks"),
//         }
//     }

//     pub(crate) fn attr(&self) -> FileAttr {
//         match self {
//             ChildDirectories::Containers => self.containers_root_attr(),
//             ChildDirectories::Images => unimplemented!(),
//             ChildDirectories::Volumes => unimplemented!(),
//             ChildDirectories::Networks => unimplemented!(),
//             ChildDirectories::Root => self.root_attr(),
//         }
//     }

//     pub(crate) async fn read_dir(
//         &self,
//         offset: i64,
//         reply: &mut fuser::ReplyDirectory,
//         docker: Arc<Mutex<super::Docker>>,
//     ) -> Result<(), libc::c_int> {
//         match self {
//             ChildDirectories::Containers => {
//                 self.containers_root_read_dir(offset, reply, docker).await
//             }
//             ChildDirectories::Images => unimplemented!(),
//             ChildDirectories::Volumes => unimplemented!(),
//             ChildDirectories::Networks => unimplemented!(),
//             ChildDirectories::Root => self.root_read_dir(offset, reply),
//         }
//     }

//     pub(crate) async fn lookup(
//         &self,
//         name: &std::ffi::OsStr,
//         reply: fuser::ReplyEntry,
//         docker: Arc<Mutex<super::Docker>>,
//     ) -> Result<(), libc::c_int> {
//         match self {
//             ChildDirectories::Containers => Self::containers_root_lookup(name, reply, docker).await,
//             ChildDirectories::Images => unimplemented!(),
//             ChildDirectories::Volumes => unimplemented!(),
//             ChildDirectories::Networks => unimplemented!(),
//             ChildDirectories::Root => Self::root_lookup(name, reply),
//         }
//     }

//     pub(crate) fn ino_from_docker_id(name: &str) -> u64 {
//         let mut ino = 0;
//         for c in name.chars().take(8) {
//             ino = ino << 8;
//             ino += c as u64;
//         }
//         ino
//     }

//     pub(crate) fn docker_id_from_ino(ino: u64) -> String {
//         let mut name = String::new();
//         for i in 0..8 {
//             let c = (ino >> (8 * (7 - i))) as u8 as char;
//             if c != '\0' {
//                 name.push(c);
//             }
//         }
//         name
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_ino_from_docker_name_and_back_short_name() {
//         let name = "test";
//         let ino = ChildDirectories::ino_from_docker_id(name);
//         let name_back = ChildDirectories::docker_id_from_ino(ino);
//         println!("{} {}", name, name_back);
//         assert_eq!(name, name_back.as_str());
//     }

//     #[test]
//     fn test_ino_from_docker_name_and_back_long_name() {
//         let name = "this_is_a_very_long_name_but_the_conversion_is_still_working";
//         let ino = ChildDirectories::ino_from_docker_id(name);
//         let name_back = ChildDirectories::docker_id_from_ino(ino);
//         println!("{} {}", name, name_back);
//         assert_eq!(
//             name.chars().take(8).collect::<String>().as_str(),
//             name_back.as_str()
//         );
//     }
// }
