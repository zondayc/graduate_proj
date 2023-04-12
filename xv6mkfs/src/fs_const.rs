use core::mem::size_of;
use crate::structs::DiskInode;
/// magic number indentifying this specific file system
pub const FSMAGIC: u32 = 0x10203040;
/// size of disk block
pub const BSIZE: usize = 512;
/// Maxinum of blocks an FS op can write
pub const MAXOPBLOCKS: usize = 10;
/// size of log space in disk
pub const LOGSIZE: usize = MAXOPBLOCKS * 3;

/// maximum number of disk inodes
pub const NDINODES: usize = 200;
/// size of file system in blocks
pub const FSSIZE: usize = 1000; 

pub const NDIRECT: usize = 11;

/// Directory is a file containing a sequence of dirent structures
pub const DIRSIZ: usize = 14;

/// Inodes per block. 
pub const IPB: usize = BSIZE / size_of::<DiskInode>();

