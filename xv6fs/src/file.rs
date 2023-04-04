use crate::SleepLock;
use crate::bitmap::inode_alloc;
use crate::disk_inode::InodeType;
use crate::fs_const::{ BSIZE, MAXOPBLOCKS };
use crate::inode::{ICACHE, self};
use super::inode::Inode;
use super::stat::Stat;
use crate::log::LOG_MANAGER;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::sync::Arc;
use axlog::info;
use axtask::spawn;
use core::sync::atomic::AtomicI32;
use crate::sync::sleeplock::init_lock;

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u16)]
pub enum FileType {
    None = 0,
    Pipe = 1,
    File = 2,
    Directory=3,
    Device = 4,
}

#[derive(Clone)]
pub struct Device {

}

#[derive(Clone)]
pub struct File {

}

#[derive(Clone)]
pub enum FileInner {
    Device(Device),
    File(File)
    // Pipe(Pipe)
}

/// Virtual File, which can abstract struct to dispatch 
/// syscall to specific file.
#[derive(Clone, Debug)]
pub struct VFile {
    pub(crate) ftype: FileType,
    pub(crate) readable: bool,
    pub(crate) writeable: bool,
    pub(crate) inode: Option<Inode>,
    pub(crate) offset: u32,
    pub(crate) major: i16
    // inner: FileInner
}

impl VFile {
    pub(crate) const fn init() -> Self {
        Self{
            ftype: FileType::None,
            readable: false,
            writeable: false,
            inode: None,
            offset: 0,
            major: 0
        }
    }

    ///addr is destination address 
    pub fn vfile_read(
        &self, 
        addr: usize,
        len: usize
    ) -> Result<usize, &'static str> {
        let ret;
        if !self.vfile_readable() {
            panic!("File can't be read!")
        }

        match self.ftype {
            FileType::File|FileType::Directory => {
                let inode = self.inode.as_ref().unwrap();
                let mut inode_guard = inode.lock();
                match inode_guard.read( addr, self.offset, len as u32) {
                    Ok(size) => {
                        ret = size;
                        let offset = unsafe { &mut *(&self.offset as *const _ as *mut u32)};
                        *offset += ret as u32;
                        drop(inode_guard);
                        Ok(ret)
                    },
                    Err(err) => {
                        Err(err)
                    }
                }
            },

            _ => {
                panic!("Invalid file!")
            },
        }
    }

    /// Write to file f. 
    /// addr is a user virtual address
    /// addr is src address
    pub fn vfile_write(
        &self, 
        addr: usize, 
        len: usize
    ) -> Result<usize, &'static str> {
        let ret; 
        if !self.vfile_writeable() {
            panic!("file can't be written")
        }
        
        match self.ftype {
            FileType::File|FileType::Directory => {
                // write a few blocks at a time to avoid exceeding 
                // the maxinum log transaction size, including
                // inode, indirect block, allocation blocks, 
                // and 2 blocks of slop for non-aligned writes. 
                // this really belongs lower down, since inode write
                // might be writing a device like console. 
                let max = ((MAXOPBLOCKS -1 -1 -2) / 2) * BSIZE;
                let mut count  = 0;
                while count < len {
                    let mut write_bytes = len - count;
                    if write_bytes > max { write_bytes = max; }

                    // start log
                    //LOG.begin_op();
                    let inode = self.inode.as_ref().unwrap();
                    let mut inode_guard = inode.lock();

                    // return err when failt to write
                    inode_guard.write(
                        addr + count, 
                        self.offset, 
                        write_bytes as u32
                    )?;

                    // release sleeplock
                    drop(inode_guard);
                    LOG_MANAGER.commit_log();
                    // end log
                    //LOG.end_op();

                    // update loop data
                    // self.offset += write_bytes as u32;
                    let offset = unsafe{ &mut *(&self.offset as *const _ as *mut u32) };
                    *offset += write_bytes as u32;
                    count += write_bytes;
                    
                }
                ret = count;
                Ok(ret)
            },

            _ => {
                panic!("Invalid File Type!")
            }
        }

    }

    fn vfile_readable(&self) -> bool {
        self.readable
    }

    fn vfile_writeable(&self) -> bool {
        self.writeable
    }

    /// Get metadata about file f. 
    /// addr is a user virtual address, pointing to a struct stat. 
    pub fn vfile_stat(&self) -> Result<Stat, &'static str> {
        let mut stat: Stat = Stat::new();
        match self.ftype {
            FileType::File|FileType::Directory => {
                let inode = self.inode.as_ref().unwrap();
                
                #[cfg(feature = "debug")]
                info!("[Kernel] stat: inode index: {}, dev: {}, inum: {}", inode.index, inode.dev, inode.inum);

                let inode_guard = inode.lock();
                inode_guard.stat(&mut stat);
                drop(inode_guard);
                
                // info!(
                //     "[Kernel] stat: dev: {}, inum: {}, nlink: {}, size: {}, type: {:?}", 
                //     stat.dev, stat.inum, stat.nlink, stat.size, stat.itype
                // );
                Ok(stat)
            },  

            _ => {
                Err("")
            }
        }
    }

    pub fn vfile_is_dir(&self)->bool{
        if self.ftype==FileType::Directory{
            return true;
        }
        false
    }

    pub fn vfile_is_file(&self)->bool{
        if self.ftype==FileType::File{
            return true;
        }
        false
    }

    pub fn vfile_open(path:&str,readable:bool,writeable:bool)->Option<Self>{
        info!("vfile open: path is {}",path);
        let inode=ICACHE.create(path.as_bytes(),crate::disk_inode::InodeType::File, 2, 1).unwrap();
        Some(Self { ftype: FileType::File, readable, writeable, inode:Some(inode), offset:0, major:2})
    }

    pub fn vfile_readdir(&self)->Option<Vec<String>>{
        if self.ftype!=FileType::Directory{
            panic!("this is not a directory!");
        }
        let mut inode_data=self.inode.as_ref().unwrap().lock();
        inode_data.ls()
    }

    pub fn vfile_remove(&self,path:&str){
        ICACHE.remove(path.as_bytes());
        LOG_MANAGER.commit_log();
    }

    pub fn vfile_create(&self,path:&str,itype:InodeType)->Self{
        info!("vfile create: path is {}",path);
        let self_inode=self.inode.as_ref().unwrap();
        let mut self_idata=self_inode.lock();
        let dev=self_inode.dev;
        let inum=inode_alloc(dev,itype);
        info!("vfile create: inum is {}",inum);
        let inode=ICACHE.get(dev, inum);
        let mut idata=inode.lock();
        idata.dinode.major=2;
        idata.dinode.minor=1;
        idata.dinode.nlink=1;
        idata.update();
        let mut ftype=FileType::File;
        if itype==InodeType::Directory{
            ftype=FileType::Directory;
            idata.dinode.nlink+=1;
            idata.update();
            idata.dir_link(".".as_bytes(), inum);
            idata.dir_link("..".as_bytes(), self_inode.inum);
        }
        self_idata.dir_link(path.as_bytes(), inode.inum).expect("parent inode fail to link");
        drop(idata);
        drop(self_idata);
        LOG_MANAGER.commit_log();
        VFile { ftype, readable:true, writeable:true, inode:Some(inode), offset:0, major:2}
        
    }

    pub fn vfile_size(&self)->usize{
        let inode=self.inode.as_ref().unwrap();
        let idata=inode.lock();
        idata.dinode.size as usize
    }

    pub fn test_sleep_lock(){
        let mut counter=0;
        let mut i=1;
        let count_lock=Arc::new(SleepLock::new(counter,init_lock()));
        let i_lock=SleepLock::new(i,init_lock());
        static SUM:AtomicI32=AtomicI32::new(0);
        for i in 0..3{
            let clock=count_lock.clone();
            axtask::spawn(move||{
                let g1=clock.lock();
                axtask::yield_now();
                drop(g1);
                info!("==========hello i===========, {}",i);
                axtask::yield_now();
                info!("hello i, {}, sum is {}",i,SUM.load(core::sync::atomic::Ordering::Acquire));
                SUM.fetch_add(1, core::sync::atomic::Ordering::Release);
            });  
        }
        loop {
            if SUM.load(core::sync::atomic::Ordering::Acquire)==3{
                break;
            }
            axtask::yield_now();
        } 
        info!("end!, sum is {}",SUM.load(core::sync::atomic::Ordering::Acquire));
    }
}





