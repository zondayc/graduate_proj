use crate::structs::*;
use crate::fs_const::*;

mod structs;
mod fs_const;

use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::mem::size_of;
use std::sync::Arc;
use std::sync::Mutex;

use std::ptr::copy_nonoverlapping;

/// Use a block size of 512 bytes
const BLOCK_SZ: usize = 512;
const BLOCK_NUM:usize = 1000;


static mut FREEBLOCK:usize=0;
static mut FREEINODE:usize=1;

struct BlockFile(Mutex<File>);

impl BlockDevice for BlockFile {
    /// Read a block from file
    fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        let mut file = self.0.lock().unwrap();
        file.seek(SeekFrom::Start((block_id * BLOCK_SZ) as u64))
            .expect("Error when seeking!");
        assert_eq!(file.read(buf).unwrap(), BLOCK_SZ, "Not a complete block!");
    }
    /// Write a block into file
    fn write_block(&self, block_id: usize, buf: &[u8]) {
        let mut file = self.0.lock().unwrap();
        file.seek(SeekFrom::Start((block_id * BLOCK_SZ) as u64))
            .expect("Error when seeking!");
        assert_eq!(file.write(buf).unwrap(), BLOCK_SZ, "Not a complete block!");
    }
}

fn iblock(inum:usize,rsb_inodestart:usize)->usize{
    inum/IPB+rsb_inodestart
}

fn ialloc(itype:InodeType)->DiskInode{
    let mut dinode=DiskInode::new();
    dinode.itype=itype;
    dinode.nlink=1;
    dinode.size=0;
    dinode.major=0;
    dinode.minor=0;
    dinode.size=512;
    unsafe{dinode.addrs[0]=FREEBLOCK as u32;}
    unsafe{FREEBLOCK+=1};
    dinode
    
}

fn main() {
    //let nbitmap= FSSIZE/(BSIZE*8) + 1;
    let ninodeblocks= NDINODES/IPB + 1;
    let nlog=LOGSIZE;
    let nmeta=2 + LOGSIZE + NDINODES/IPB + 1 + FSSIZE/(BSIZE*8) + 1;
    let nblocks= FSSIZE-(2 + LOGSIZE + NDINODES/IPB + 1 + FSSIZE/(BSIZE*8) + 1);
    //init superblock
    let mut raw_superblock=RawSuperBlock::new();
    raw_superblock.magic=FSMAGIC;
    raw_superblock.size=FSSIZE as u32;
    raw_superblock.nblocks=nblocks as u32;
    raw_superblock.ninodes=NDINODES as u32;
    raw_superblock.nlog=nlog as u32;
    raw_superblock.logstart=2;
    raw_superblock.inodestart=2+nlog as u32;
    raw_superblock.bmapstart=(2+30+ninodeblocks) as u32;

    //memset disk to 0
    let mut buf=[0 as u8;BSIZE];
    let block_file = Arc::new(BlockFile(Mutex::new({
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("target/disk.img").unwrap();
        f.set_len((BLOCK_NUM * BLOCK_SZ) as u64).unwrap();
        f
    })));
    for i in 0..FSSIZE{
        block_file.write_block(i, &buf);
    }

    //write superblock
    unsafe{copy_nonoverlapping(&raw_superblock as *const RawSuperBlock, buf.as_mut_ptr() as *mut RawSuperBlock, 1);}
    block_file.write_block(1, &buf);

    //set root inode
    unsafe{FREEBLOCK=nmeta;}
    let rinum:usize=1;
    unsafe{FREEINODE+=1;}
    let dinode=ialloc(InodeType::Directory);
    println!("rinum is {}, and dinode.addr[0] is {}",rinum,dinode.addrs[0]);
    let block_id=iblock(rinum, raw_superblock.inodestart as usize);
    println!("blockid is {}",block_id);
    println!("dinode is {:?}",dinode);
    block_file.read_block(block_id, &mut buf);
    //println!("buf is {:?}",buf);
    unsafe{
        copy_nonoverlapping(
            &dinode as *const DiskInode, 
            (buf.as_mut_ptr() as usize + (rinum%IPB)*core::mem::size_of::<DiskInode>()) as *mut DiskInode, 
            1
        );
    }
    //println!("buf is {:?}",buf);
    block_file.write_block(block_id, &buf);

    //write direct entry 
    let mut dir_entry=DirEntry::new();
    dir_entry.name[0]=".".as_bytes()[0];
    dir_entry.inum=1;
    let block_id=dinode.addrs[0];
    block_file.read_block(block_id as usize, &mut buf);
    unsafe{
        copy_nonoverlapping(
            &dir_entry as *const DirEntry,
            buf.as_mut_ptr() as usize as *mut DirEntry,
            1
        );
    }
    block_file.write_block(block_id as usize, &buf);
    let mut dir_entry=DirEntry::new();
    dir_entry.name[0]="..".as_bytes()[0];
    dir_entry.name[1]="..".as_bytes()[1];
    dir_entry.inum=1;
    let block_id=dinode.addrs[0];
    block_file.read_block(block_id as usize, &mut buf);
    unsafe{
        copy_nonoverlapping(
            &dir_entry as *const DirEntry,
            (buf.as_mut_ptr() as usize + size_of::<DirEntry>()) as *mut DirEntry,
            1
        );
    }
    block_file.write_block(block_id as usize, &buf);

    //write bitmap
    let bblock_id=raw_superblock.bmapstart;
    println!("bitmap start is {}",bblock_id);
    let used=unsafe{FREEBLOCK};
    println!("used block is {}",used);
    block_file.read_block(bblock_id as usize, &mut buf);
    for i in 0..used{
        buf[i/8]=buf[i/8]|(0x1 << (i%8));
    }
    block_file.write_block(bblock_id as usize, &buf);
}
