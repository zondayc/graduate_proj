use crate::fs_const::*;
use core::any::Any;
pub struct RawSuperBlock {
    pub magic: u32,      // Must be FSMAGIC
    pub size: u32,       // Size of file system image (blocks)
    pub nblocks: u32,    // Number of data blocks
    pub ninodes: u32,    // Number of inodes
    pub nlog: u32,       // Number of log blocks
    pub logstart: u32,   // Block number of first log block
    pub inodestart: u32, // Block number of first inode block
    pub bmapstart: u32,  // Block number of first free map block
}

impl RawSuperBlock {
    pub fn new()->Self{
        RawSuperBlock { magic: 0, size: 0, nblocks: 0, ninodes: 0, 
            nlog: 0, logstart: 0, inodestart: 0, bmapstart: 0 }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum InodeType {
    Empty = 0,
    Directory = 1,
}

/// On-disk inode structure
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct DiskInode {
    pub itype: InodeType, // File type
    pub major: i16, // Major device number (T_REVICE only)
    pub minor: i16, // Minor device number (T_DEVICE only)
    pub nlink: i16, // Number of links to inode in file system
    pub size: u32, // Size of file (bytes)
    pub addrs: [u32; NDIRECT+2] // Data block addresses
}

impl DiskInode {
    pub const fn new() -> Self {
        Self {
            itype: InodeType::Empty,
            major: 0,
            minor: 0,
            nlink: 0,
            size: 0,
            addrs: [0; NDIRECT+2]
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct DirEntry {
    pub inum: u16,
    pub name:[u8;DIRSIZ]
}

impl DirEntry {
    pub const fn new() -> Self {
        Self {
            inum: 0,
            name: [0;DIRSIZ]
        }
    }
}

pub trait BlockDevice : Send + Sync + Any {
    fn read_block(&self, _block_id: usize, _buf: &mut [u8]);
    fn write_block(&self, _block_id: usize, _buf: &[u8]);
}