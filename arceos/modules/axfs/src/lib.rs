#![no_std]

pub mod mount;
mod ops;
pub mod sleeplock_shim;

#[macro_use]
extern crate alloc;
#[macro_use]
extern crate axlog;

use alloc::{sync::Arc, vec::Vec};
use axdriver::{block_devices, BlockDevices};
use driver_block::BlockDriverOps;
use fatfs_shim::Fat32FileSystem;
use sleeplock_shim::FsLockList;
use spin::mutex::Mutex;
use xv6fs::interface::INTERFACE_MANAGER;
use xv6fs_shim::{VXV6FS};
use xv6fs::{BlockDevice, interface::FsInterface};
use lazy_init::LazyInit;
use vfscore::{DiskOperation, VfsFileSystem};

use crate::mount::{MountedFsList, MOUNTEDFS};
use crate::sleeplock_shim::{FS_LOCK_LIST};
pub use ops::*;

static FILESTSTEMS: LazyInit<FileSystemList> = LazyInit::new();
static BLOCK_DEV:LazyInit<BlockDevices>=LazyInit::new();

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

// pub fn init_filesystems() {
//     info!("init filesystems");
//     // init filesystems
//     let fat32 = Arc::new(Fat32FileSystem::<DiskOps>::new());

//     // init filesystem list
//     let mut fs_list = FileSystemList::new();
//     fs_list.add(fat32.clone());
//     FILESTSTEMS.init_by(fs_list);

//     // init mounted filesystem list
//     let mut mounted_list = MountedFsList::new();
//     mounted_list.mount("/", fat32.clone());
//     MOUNTEDFS.init_by(mounted_list)
// }

pub fn init_filesystems(blk_devs: BlockDevices) {
    info!("init block device");
    init_block_dev(blk_devs);
    info!("init xv6fs");
    let xfs=Arc::new(VXV6FS::new());
    unsafe{xv6fs::init(Arc::new(DiskOps), 0);}
    INTERFACE_MANAGER.lock().set_interface(Arc::new(AxFsInterface));

    let lock_list=Mutex::new(FsLockList::new());
    FS_LOCK_LIST.init_by(lock_list);

    // init filesystem list
    let mut fs_list = FileSystemList::new();
    fs_list.add(xfs.clone());
    FILESTSTEMS.init_by(fs_list);

    // init mounted filesystem list
    let mut mounted_list = MountedFsList::new();
    mounted_list.mount("/", xfs.clone());
    MOUNTEDFS.init_by(mounted_list)

}

fn init_block_dev(blk_devs: BlockDevices){
    BLOCK_DEV.init_by(blk_devs);
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

impl BlockDevice for DiskOps {
    fn read_block(&self,index: usize, buf: &mut [u8]) {
        info!("read blockdevice");
        BLOCK_DEV
            .0
            .read_block(index, buf)
            .expect("can't read block");
    }

    fn write_block(&self,index: usize, data: &[u8]) {
        BLOCK_DEV
            .0
            .write_block(index, data)
            .expect("can't write block");
    }
}

pub fn filesystems() -> &'static FileSystemList {
    &FILESTSTEMS
}

pub struct AxFsInterface;

impl FsInterface for AxFsInterface{
    fn get_cur_dir_inode(&self)->Option<xv6fs::inode::Inode> {
        None
    }
    fn new_sleep_lock(&self)->usize {
        FS_LOCK_LIST.lock().new_lock()
    }
    fn sleep_cur_proc(&self,index:usize) {
        info!("index is {}",index);
        info!("before, lock is {}",FS_LOCK_LIST.lock().lock_list[index].flag.load(core::sync::atomic::Ordering::Acquire));
        FS_LOCK_LIST.lock().lock_list[index].sleep_cur_task();
        info!("after, lock is {}",FS_LOCK_LIST.lock().lock_list[index].flag.load(core::sync::atomic::Ordering::Acquire));
    }
    fn wake_up_next_proc(&self,index:usize) {
        FS_LOCK_LIST.lock().lock_list[index].wake_up_next_proc();
    }
    fn get_flag(&self,index:usize)->bool {
        FS_LOCK_LIST.lock().lock_list[index].flag.load(core::sync::atomic::Ordering::Acquire)
    }
}

pub fn test_sleep_lock(){
    xv6fs::file::VFile::test_sleep_lock();
}