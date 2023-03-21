#![no_std]

pub mod mount;
mod ops;

#[macro_use]
extern crate alloc;
#[macro_use]
extern crate axlog;

use alloc::{sync::Arc, vec::Vec};
use axdriver::block_devices;
use driver_block::BlockDriverOps;
use fatfs_shim::Fat32FileSystem;
use lazy_init::LazyInit;
use vfscore::{DiskOperation, VfsFileSystem};

use crate::mount::{MountedFsList, MOUNTEDFS};
pub use ops::*;

static FILESTSTEMS: LazyInit<FileSystemList> = LazyInit::new();

pub struct FileSystemList(Vec<Arc<dyn VfsFileSystem>>);

impl FileSystemList {
    pub(crate) const fn new() -> Self {
        Self(vec![])
    }

    pub(crate) fn add(&mut self, fs: Arc<dyn VfsFileSystem>) {
        // info!(
        //     "Added new {} filesystem",
        //     fs.as_ref().name()
        // );
        self.0.push(fs);
    }

    pub fn first(&self) -> Option<&Arc<dyn VfsFileSystem>> {
        self.0.first()
    }
}

pub fn init_filesystems() {
    info!("init filesystems");
    // init filesystems
    let fat32 = Arc::new(Fat32FileSystem::<DiskOps>::new());

    // init filesystem list
    let mut fs_list = FileSystemList::new();
    fs_list.add(fat32.clone());
    FILESTSTEMS.init_by(fs_list);

    // init mounted filesystem list
    let mut mounted_list = MountedFsList::new();
    mounted_list.mount("/", fat32.clone());
    MOUNTEDFS.init_by(mounted_list)
}

pub struct DiskOps;

impl DiskOperation for DiskOps {
    fn read_block(index: usize, buf: &mut [u8]) {
        block_devices()
            .0
            .read_block(index, buf)
            .expect("can't read block");
    }

    fn write_block(index: usize, data: &[u8]) {
        block_devices()
            .0
            .write_block(index, data)
            .expect("can't write block");
    }
}

pub fn filesystems() -> &'static FileSystemList {
    &FILESTSTEMS
}
