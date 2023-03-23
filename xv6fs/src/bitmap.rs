#[cfg(not(test))]
use axlog::{info, warn}; // Use log crate when building application
 
#[cfg(test)]
use std::{println as info, println as warn}; // Workaround to use prinltn! for logs.

use bit_field::BitField;

use crate::superblock::SUPER_BLOCK;
use crate::log::LOG_MANAGER;
use crate::buffer_cache::BLOCK_CACHE_MANAGER;
use super::{ InodeType, DiskInode };


use crate::fs_const::{ BPB,IPB };

use core::ptr;

// / Zero a block. 
// pub fn bzero(dev: u32, bno: u32) {
//     let mut buf = BCACHE.bread(dev, bno);
//     unsafe{ (&mut *buf.raw_data_mut()).zero() };
//     LOG.write(buf);
// }

/// Given an inode number. 
/// Calculate the offset index of this inode inside the block. 
#[inline]
fn locate_inode_offset(inum: u32) -> usize {
    inum as usize % IPB
}

/// Free a block in the disk by setting the relevant bit in bitmap to 0.
// pub fn bfree(dev: u32, blockno: u32) {
//     let bm_blockno = unsafe { SUPER_BLOCK.bitmap_blockno(blockno) };
//     let bm_offset = blockno % BPB;
//     let index = (bm_offset / 8) as isize;
//     let bit = (bm_offset % 8) as usize;
//     let mut buf = BLOCK_CACHE_MANAGER.bread(dev, bm_blockno);
    
//     let byte = unsafe { (buf.raw_data_mut() as *mut u8).offset(index).as_mut().unwrap() };
//     if !byte.get_bit(bit) {
//         panic!("bitmap: double freeing a block");
//     }
//     byte.set_bit(bit, false);
//     LOG_MANAGER.write(buf);
// }


/// Allocate a zeroed disk block 
pub fn balloc(dev: u32) -> u32 {
    let mut b = 0;
    let sb_size = unsafe{ SUPER_BLOCK.size() };
    while b < sb_size {
        let bm_blockno = unsafe{ SUPER_BLOCK.bitmap_blockno(b) };
        let mut buf = BLOCK_CACHE_MANAGER.bread(dev, bm_blockno);
        let mut bi = 0;
        while bi < BPB && b + bi < sb_size {
            bi += 1;
            let m = 1 << (bi % 8);
            let buf_ptr = unsafe{ (buf.raw_data_mut() as *mut u8).offset((bi / 8) as isize).as_mut().unwrap() };
            let buf_val = unsafe{ ptr::read(buf_ptr) };
            //info!("bval is {}",buf_val);
            if (buf_val&m) == 0{ // Is block free?
                let new_val:u8=buf_val|m;
                unsafe{ ptr::write(buf_ptr, new_val) };
                //info!("balloc: inum is {}",bi);
                LOG_MANAGER.write(buf);
                // drop(buf);
                // bzero(dev, b + bi);
                return b + bi
            }
        }
        drop(buf);
        b += BPB;
    }
    panic!("balloc: out of the block ranges.")
}

pub fn bfree(devno:u32,blockno:u32)->Result<(),&'static str>{
    let bm_blockno=unsafe {SUPER_BLOCK.bitmap_blockno(blockno)};
    let mut buf=BLOCK_CACHE_MANAGER.bread(0, bm_blockno);
    let bi=blockno%8;
    let offset=blockno/8;
    let buf_ptr=unsafe {(buf.raw_data_mut() as *mut u8).offset(offset as isize).as_mut().unwrap()};
    let buf_val=unsafe {ptr::read(buf_ptr)};
    info!("buf val is {}",buf_val);
    if buf_val&(1<<bi)==0{
        panic!("this bit is not alloc yet");
    }
    let new_val=buf_val^(1<<bi);
    info!("new val is {}",new_val);
    unsafe{ptr::write(buf_ptr, new_val)};
    //unsafe{info!("buf is {:?}",buf.raw_data().as_ref().unwrap())};
    LOG_MANAGER.write(buf);
    Ok(())
}

pub fn inode_alloc(dev: u32, itype: InodeType) -> u32 {
    let size = unsafe { SUPER_BLOCK.ninodes() };
    for inum in 1..size {
        let blockno = unsafe { SUPER_BLOCK.locate_inode(inum) };
        let offset = locate_inode_offset(inum) as isize;
        let mut buf = BLOCK_CACHE_MANAGER.bread(dev, blockno);
        let dinode = unsafe { (buf.raw_data_mut() as *mut DiskInode).offset(offset) };
        let dinode = unsafe { &mut *dinode };
        if dinode.try_alloc(itype).is_ok() {
            info!("inode alloc: inum is {} and offset is {}",inum,offset);
            LOG_MANAGER.write(buf);
            info!("inum is {}",inum);
            return inum
        }
    }

    panic!("not enough inode to alloc");
}