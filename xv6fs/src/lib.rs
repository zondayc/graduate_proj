//#![no_std]

extern crate alloc;

pub mod block_dev;
pub mod fs_const;
pub mod buffer_cache;
pub mod log;
pub mod superblock;
pub mod stat;
pub mod disk_inode;
pub mod bitmap;
pub mod inode;
pub mod misc;
pub mod file;
pub mod interface;
pub mod sync;
pub mod xv6fs;

use core::ops::DerefMut;
use std::println;

use alloc::sync::Arc;
pub use block_dev::BlockDevice;
use buffer_cache::BLOCK_CACHE_MANAGER;
use fs_const::{NBUF,BSIZE};
use disk_inode::{InodeType,DiskInode};
use log::{LOG_MANAGER,Log};
use superblock::SUPER_BLOCK;
use xv6fs::Xv6FileSystem;

pub unsafe fn init(block_dev:Arc<dyn BlockDevice>,dev:u32) {
    BLOCK_CACHE_MANAGER.set_block_device(Arc::clone(&block_dev));
    BLOCK_CACHE_MANAGER.binit();
    SUPER_BLOCK.init(dev);
    let log=LOG_MANAGER.log.lock().deref_mut() as *mut Log;
    log.as_mut().unwrap().init(dev);
    println!("file system: setup done!");
}