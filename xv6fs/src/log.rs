//! Log-relevant operations

//use core::{ops::{Deref, DerefMut}, panic, ptr};
use core::{ panic, ptr};
use core::mem;
//use alloc::sync::Arc;
use spin::Mutex;
use lazy_static::*;

//use crate::{fs_const::{MAXOPBLOCKS, LOGSIZE, BSIZE}, block_dev::BlockDevice};
use crate::fs_const::{LOGSIZE, BSIZE};
use crate::buffer_cache::{BLOCK_CACHE_MANAGER, Buf, BufData};
//use crate::block_dev::BlockDevice;
use crate::superblock::SUPER_BLOCK;

lazy_static!{
    pub static ref LOG_MANAGER: LogManager = LogManager::init();
}

pub struct LogManager{
    pub log: Mutex<Log>,
}

impl LogManager {
    pub fn init()->Self{
        LogManager { log: Mutex::new(Log::uninit()) }
    }
}

/// Log info about the file system.
pub struct Log {
    /// the starting block in the fs
    start: u32,
    /// the number of blocks available for log
    size: u32,
    dev: u32,
    outstanding: u32,
    /// not allow any fs op when the log is committing
    committing: bool,
    lh: LogHeader,
}

impl Log {
    const fn uninit() -> Self {
        Self {
            start: 0,
            size: 0,
            dev: 0,
            outstanding: 0,
            committing: false,
            lh: LogHeader { len: 0, blocknos: [0; LOGSIZE-1] },
        }
    }

    /// Init the log when booting.
    /// Recover the fs if necessary.
    /// SAFETY: It must be called without holding any locks,
    ///         because it will call disk rw, which might sleep.
    /// 这里的dev要再考虑一下
    pub unsafe fn init(&mut self, dev: u32) {
        debug_assert!(mem::size_of::<LogHeader>() < BSIZE);
        debug_assert_eq!(mem::align_of::<BufData>() % mem::align_of::<LogHeader>(), 0);
        let (start, size) = SUPER_BLOCK.read_log();
        self.start = start;
        self.size = size;
        self.dev = dev;
        self.recover();
    }

    /// Recover the file system from log if necessary.
    fn recover(&mut self) {
        //println!("file system: checking logs");
        self.read_head();
        if self.lh.len > 0 {
            //println!("file system: recovering from logs");
            self.install_trans(true);
            self.empty_head();
        } else {
            //println!("file system: no need to recover");
        }
    }

    /// Read the log header from disk into the in-memory log header.
    fn read_head(&mut self) {
        let buf = BLOCK_CACHE_MANAGER.bread(self.dev, self.start);
        unsafe {
            ptr::copy_nonoverlapping(
                buf.raw_data() as *const LogHeader,
                &mut self.lh,
                1
            );
        }
        drop(buf);
    }

    /// Write in-memory log header to disk.
    /// This is the true point at which the current transaction commits.
    fn write_head(&mut self) {
        let mut buf = BLOCK_CACHE_MANAGER.bread(self.dev, self.start);
        unsafe {
            ptr::copy_nonoverlapping(
                &self.lh,
                buf.raw_data_mut() as *mut LogHeader,
                1,
            );
        }
        buf.bwrite();
        drop(buf);
    }

    /// Empty log header in disk by 
    /// setting the len of log(both in-memory and in-disk) to zero.
    fn empty_head(&mut self) {
        self.lh.len = 0;
        let mut buf = BLOCK_CACHE_MANAGER.bread(self.dev, self.start);
        let raw_lh = buf.raw_data_mut() as *mut LogHeader;
        unsafe { raw_lh.as_mut().unwrap().len = 0; }
        buf.bwrite();
        drop(buf);
    }

    /// Copy committed blocks from log to their home location.
    fn install_trans(&mut self, recovering: bool) {
        for i in 0..self.lh.len {
            let log_buf  = BLOCK_CACHE_MANAGER.bread(self.dev, self.start+1+i);
            let mut disk_buf = BLOCK_CACHE_MANAGER.bread(self.dev, self.lh.blocknos[i as usize]);
            unsafe {
                ptr::copy(
                    log_buf.raw_data(),
                    disk_buf.raw_data_mut(),
                    1,
                );
            }
            disk_buf.bwrite();
            if !recovering {
                println!("unpin disk buf {}",self.lh.blocknos[i as usize]);
                unsafe { disk_buf.unpin(); }
            }
            drop(log_buf);
            drop(disk_buf);
        }
    }

    /// Commit the log.
    /// SAFETY: It must be called while the committing field is set.
    pub unsafe fn commit(&mut self) {
        self.committing=true;
        if !self.committing {
            panic!("log: committing while the committing flag is not set");
        }
        // debug_assert!(self.lh.len > 0);     // it should have some log to commit
        if self.lh.len > 0 {
            self.write_log();
            self.write_head();
            self.install_trans(false);
            self.empty_head();
        }
        self.committing=false;
    }

    /// Copy the log content from buffer cache to disk.
    fn write_log(&mut self) {
        for i in 0..self.lh.len {
            let mut log_buf  = BLOCK_CACHE_MANAGER.bread(self.dev, self.start+1+i);
            let cache_buf = BLOCK_CACHE_MANAGER.bread(self.dev, self.lh.blocknos[i as usize]);
            unsafe {
                ptr::copy(
                    cache_buf.raw_data(),
                    log_buf.raw_data_mut(),
                    1,
                );
            }
            log_buf.bwrite();
            drop(cache_buf);
            drop(log_buf);
        }
    }
}

impl LogManager {
    /// It should be called at the start of file system call.
    // pub fn begin_op(&self) {
    //     let mut guard  = self.acquire();
    //     loop {
    //         if guard.committing ||
    //             1 + guard.lh.len as usize + (guard.outstanding+1) as usize * MAXOPBLOCKS > LOGSIZE
    //         {
    //             let channel = guard.deref() as *const Log as usize;
    //             unsafe { CPU_MANAGER.myproc().unwrap().sleep(channel, guard); }
    //             guard = self.acquire();
    //         } 
    //         else 
    //         {
    //             guard.outstanding += 1;
    //             drop(guard);
    //             break;
    //         }
    //     }
    // }

    /// Accept a buffer, write it into the log and then release the buffer.
    /// This function will pin this buf in the cache until the log commits.
    pub fn write(&self, buf: Buf) {
        let mut guard = self.log.lock();
        
        if (guard.lh.len+1) as usize >= LOGSIZE || guard.lh.len+1 >= guard.size {
            panic!("log: not enough space for ongoing transactions");
        }
        // if guard.outstanding < 1 {
        //     panic!("log: this log write is out of recording");
        // }

        // record the buf's blockno in the log header
        for i in 0..guard.lh.len {
            if guard.lh.blocknos[i as usize] == buf.read_blockno() {
                println!("buf blockno {} is in the lh.blocknos, and now len is {}",guard.lh.blocknos[i as usize],guard.lh.len);
                drop(guard);
                drop(buf);
                return;
            }
        }
        if (guard.lh.len+2) as usize >= LOGSIZE || guard.lh.len+2 >= guard.size {
            panic!("log: not enough space for this transaction");
        }
        unsafe { buf.pin(); }
        let len = guard.lh.len as usize;
        guard.lh.blocknos[len] = buf.read_blockno();
        guard.lh.len += 1;
        println!("insert blockno {},Log Header len +1, and now len is {}",buf.read_blockno(),guard.lh.len);
        drop(guard);
        drop(buf);
    }

    // It should be called at the end of file system call.
    // It will commit the log if this is the last outstanding op.
    pub fn commit_log(&self) {
        let mut guard = self.log.lock();
        unsafe{guard.commit()};
        drop(guard);
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct LogHeader {
    len: u32,                       // current len of blocknos array
    blocknos: [u32; LOGSIZE-1],     // LOGSIZE-1: one block left for log info
}
