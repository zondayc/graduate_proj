use alloc::{
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};
use lazy_init::LazyInit;
use vfscore::VfsFileSystem;

pub(crate) static MOUNTEDFS: LazyInit<MountedFsList> = LazyInit::new();

/// Mounted filesystem
#[derive(Clone)]
pub struct MountedFileSystem {
    path: String,
    fs: Arc<dyn VfsFileSystem>,
}

impl MountedFileSystem {
    pub const fn new(path: String, fs: Arc<dyn VfsFileSystem>) -> Self {
        MountedFileSystem { path, fs }
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn fs(&self) -> &Arc<dyn VfsFileSystem> {
        &self.fs
    }
}

// TODO: mount or umount through the MountedFileSystem
// pub trait Mount {
//     fn mount(&mut self, path: &str) -> Result<(), String>;
//     fn umount(&mut self) -> Result<(), String>;
// }
// impl Mount for MountedFileSystem {
//     fn mount(&mut self, path: &str) -> Result<(), alloc::string::String> {
//         todo!("mount a file system")
//     }

//     fn umount(&mut self) -> Result<(), String> {
//         // MOUNTEDFS.0.drain_filter(filter)
//         MOUNTEDFS.umount(self);
//         Ok(())
//     }
// }

/// Mounted file system list
pub struct MountedFsList(Vec<MountedFileSystem>);

impl MountedFsList {
    pub fn new() -> Self {
        Self(vec![])
    }

    /// umount a file system
    pub fn umount(&mut self, mounted_fs: &MountedFileSystem) {
        self.0
            .retain(|fs| fs.path == mounted_fs.path && fs.fs.name() == mounted_fs.fs.name());
    }

    /// umount a file system by path
    pub fn umount_by_path(&mut self, path: &str) {
        self.0.retain(|fs| fs.path == path);
    }

    /// umount a file system by fs
    pub fn umount_by_fs(&mut self, fs: Arc<dyn VfsFileSystem>) {
        self.0.retain(|mfs| mfs.fs.name() == fs.name());
    }

    /// mount a file system
    pub fn mount(&mut self, path: &str, source_fs: Arc<dyn VfsFileSystem>) {
        // find whether it was mounted.
        let search = self
            .0
            .iter()
            .find(|fs| fs.path == path && fs.fs.name() == source_fs.name());

        // mount if it not been mounted.
        if search.is_none() {
            self.0
                .push(MountedFileSystem::new(path.to_string(), source_fs.clone()))
        }
    }

    /// get a matched fiel system by path
    pub fn get_matched_fs(&self, path: &str) -> Option<&MountedFileSystem> {
        let (_, index) = self
            .0
            .iter()
            .enumerate()
            .fold((0, usize::MAX), |(len, index), (i, x)| {
                if x.path.len() >= len && path.starts_with(&x.path) {
                    (x.path.len(), i)
                } else {
                    (len, index)
                }
            });

        // return matched fs
        match index {
            usize::MAX => None,
            index => Some(self.0.get(index).unwrap()),
        }
    }
}
