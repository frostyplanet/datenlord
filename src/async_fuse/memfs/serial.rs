use std::path::PathBuf;
use std::time::SystemTime;

use nix::sys::stat::SFlag;
use serde::{Deserialize, Serialize};

use super::fs_util::FileAttr;
use super::s3_node::S3NodeData;
use crate::async_fuse::fuse::protocol::INum;

/// Serializable `FileAttr`
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct SerialFileAttr {
    /// Inode number
    ino: INum,
    /// Size in bytes
    size: u64,
    /// Size in blocks
    blocks: u64,
    /// Time of last access
    atime: SystemTime,
    /// Time of last modification
    mtime: SystemTime,
    /// Time of last change
    ctime: SystemTime,
    /// Kind of file (directory, file, pipe, etc)
    kind: SerialSFlag,
    /// Permissions
    perm: u16,
    /// Number of hard links
    nlink: u32,
    /// User id
    uid: u32,
    /// Group id
    gid: u32,
    /// Rdev
    rdev: u32,
    /// Serial node version number, default is 0 for compatibility
    #[serde(default)]
    version: u64,
}

impl SerialFileAttr {
    #[must_use]
    /// Get the inode number
    pub fn get_ino(&self) -> INum {
        self.ino
    }
}

/// Serializable `SFlag`
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub enum SerialSFlag {
    /// Regular file
    Reg,
    /// Directory
    Dir,
    /// Symbolic link
    Lnk,
}

/// In order to derive Serialize and Deserialize,
/// Replace the `BTreeMap`<String, `DirEntry`>' with `HashMap`<String,
/// `SerialDirEntry`>'
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub enum SerialNodeData {
    /// Directory data
    Directory,
    /// File data is ignored ,because `Arc<GlobalCache>` is not serializable
    File,
    /// Symbolic link data
    SymLink(PathBuf),
}

impl SerialNodeData {
    /// Deserializes the node data
    #[must_use]
    pub fn into_s3_nodedata(self) -> S3NodeData {
        match self {
            SerialNodeData::Directory => S3NodeData::Directory,
            SerialNodeData::File => S3NodeData::RegFile,
            SerialNodeData::SymLink(path) => S3NodeData::SymLink(path),
        }
    }
}
/// TODO: We should discuss the design about persist
/// Serializable 'Node'
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct SerialNode {
    /// Parent node i-number
    pub(crate) parent: u64,
    /// S3Node name
    pub(crate) name: String,
    /// Node attribute
    pub(crate) attr: SerialFileAttr,
    /// Node data
    pub(crate) data: SerialNodeData,
    /// S3Node lookup counter
    pub(crate) lookup_count: i64,
    /// If S3Node has been marked as deferred deletion
    pub(crate) deferred_deletion: bool,
}

/// Convert `SFlag` to `SerialSFlag`
#[must_use]
pub fn entry_type_to_serial(entry_type: SFlag) -> SerialSFlag {
    match entry_type {
        SFlag::S_IFDIR => SerialSFlag::Dir,
        SFlag::S_IFREG => SerialSFlag::Reg,
        SFlag::S_IFLNK => SerialSFlag::Lnk,
        _ => panic!("unsupported entry type {entry_type:?}"),
    }
}

/// Convert `SerialSFlag` to `SFlag`
#[must_use]
pub const fn serial_to_entry_type(entry_type: &SerialSFlag) -> SFlag {
    match *entry_type {
        SerialSFlag::Dir => SFlag::S_IFDIR,
        SerialSFlag::Reg => SFlag::S_IFREG,
        SerialSFlag::Lnk => SFlag::S_IFLNK,
    }
}

/// Convert `FileAttr` to `SerialFileAttr`
#[must_use]
pub fn file_attr_to_serial(attr: &FileAttr) -> SerialFileAttr {
    SerialFileAttr {
        ino: attr.ino,
        size: attr.size,
        blocks: attr.blocks,
        atime: attr.atime,
        mtime: attr.mtime,
        ctime: attr.ctime,
        kind: {
            if attr.kind == SFlag::S_IFREG {
                SerialSFlag::Reg
            } else if attr.kind == SFlag::S_IFDIR {
                SerialSFlag::Dir
            } else {
                SerialSFlag::Lnk
            }
        },
        perm: attr.perm,
        nlink: attr.nlink,
        uid: attr.uid,
        gid: attr.gid,
        rdev: attr.rdev,
        version: attr.version,
    }
}

/// Convert `SerialFileAttr` to `FileAttr`
#[must_use]
pub const fn serial_to_file_attr(attr: &SerialFileAttr) -> FileAttr {
    FileAttr {
        ino: attr.ino,
        size: attr.size,
        blocks: attr.blocks,
        atime: attr.atime,
        mtime: attr.mtime,
        ctime: attr.ctime,
        kind: {
            match attr.kind {
                SerialSFlag::Lnk => SFlag::S_IFLNK,
                SerialSFlag::Dir => SFlag::S_IFDIR,
                SerialSFlag::Reg => SFlag::S_IFREG,
            }
        },
        perm: attr.perm,
        nlink: attr.nlink,
        uid: attr.uid,
        gid: attr.gid,
        rdev: attr.rdev,
        version: attr.version,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod test {

    use super::*;

    #[test]
    fn test_entrytype_serialize() {
        use nix::sys::stat::SFlag;

        /// Test `entry_type_to_serial` and `serial_to_entry_type`
        /// `entry_type_to_serial` and `serial_to_entry_type` should be a pair
        /// of inverse functions We will test all the possible entry
        /// types (currently just three types)
        use super::entry_type_to_serial;
        use super::serial_to_entry_type;
        let entry_types = vec![SFlag::S_IFDIR, SFlag::S_IFREG, SFlag::S_IFLNK];
        for entry_type_before in entry_types {
            let serial_entry_type = entry_type_to_serial(entry_type_before);
            let entry_type_after = serial_to_entry_type(&serial_entry_type);
            assert_eq!(entry_type_before, entry_type_after);
        }
    }

    use std::time::SystemTime;

    use rand::Rng;

    // Helper function to create a FileAttr instance for testing
    fn create_file_attr() -> FileAttr {
        let mut rng = rand::thread_rng();

        FileAttr {
            ino: rng.gen(),
            size: rng.gen(),
            blocks: rng.gen(),
            atime: SystemTime::now(),
            mtime: SystemTime::now(),
            ctime: SystemTime::now(),
            kind: SFlag::S_IFREG,
            perm: rng.gen(),
            nlink: rng.gen(),
            uid: rng.gen(),
            gid: rng.gen(),
            rdev: rng.gen(),
            version: rng.gen(),
        }
    }

    // Test for file_attr_to_serial function
    #[test]
    fn test_file_attr_to_serial() {
        let file_attr = create_file_attr();
        let serial_file_attr = file_attr_to_serial(&file_attr);

        assert_eq!(file_attr.ino, serial_file_attr.ino);
        assert_eq!(file_attr.size, serial_file_attr.size);
        assert_eq!(file_attr.blocks, serial_file_attr.blocks);
        assert_eq!(file_attr.atime, serial_file_attr.atime);
        assert_eq!(file_attr.mtime, serial_file_attr.mtime);
        assert_eq!(file_attr.ctime, serial_file_attr.ctime);
        // test if the sesrial_file_attr's kind is SerialSFlag::Reg
        if let SerialSFlag::Reg = serial_file_attr.kind {
        } else {
            panic!("serial_file_attr's kind should be SerialSFlag::Reg");
        }
        assert_eq!(file_attr.perm, serial_file_attr.perm);
        assert_eq!(file_attr.nlink, serial_file_attr.nlink);
        assert_eq!(file_attr.uid, serial_file_attr.uid);
        assert_eq!(file_attr.gid, serial_file_attr.gid);
        assert_eq!(file_attr.rdev, serial_file_attr.rdev);
    }

    // Return true for equal
    fn fileattr_equal(left: &FileAttr, right: &FileAttr) -> bool {
        left.ino == right.ino
            && left.size == right.size
            && left.blocks == right.blocks
            && left.atime == right.atime
            && left.mtime == right.mtime
            && left.ctime == right.ctime
            && left.kind == right.kind
            && left.perm == right.perm
            && left.nlink == right.nlink
            && left.uid == right.uid
            && left.gid == right.gid
            && left.rdev == right.rdev
    }

    // Test for serial_to_file_attr function
    #[test]
    fn test_serial_to_file_attr() {
        let file_attr = create_file_attr();
        let serial_file_attr = file_attr_to_serial(&file_attr);
        let converted_file_attr = serial_to_file_attr(&serial_file_attr);
        assert!(fileattr_equal(&file_attr, &converted_file_attr));
    }
}
