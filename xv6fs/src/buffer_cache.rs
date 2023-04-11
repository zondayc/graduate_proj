//! buffer cache layer

use axlog::debug;
#[cfg(not(test))]
use axlog::{info, warn}; // Use log crate when building application
 
#[cfg(test)]
use std::{println as info, println as warn}; // Workaround to use prinltn! for logs.

use array_macro::array;

use core::ptr;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{Ordering, AtomicBool};

use spin::{Mutex, MutexGuard};
use crate::{SleepLock, SleepLockGuard, init_lock};
use crate::block_dev::BlockNone;

use super::{BlockDevice,NBUF, BSIZE};
use alloc::sync::Arc;
use lazy_static::*;
use crate::sync::UPSafeCell;
// lazy_static!{
//     pub static ref BLOCK_CACHE_MANAGER: Mutex<BlockCacheManager> = Mutex::new(
//         BlockCacheManager::new());
// }

lazy_static!{
    pub static ref BLOCK_CACHE_MANAGER:BlockCacheManager=BlockCacheManager::new();
}

pub struct BlockCacheManager {
    ctrl: Mutex<BufLru>,
    bufs: [BufInner; NBUF],
    inner: UPSafeCell<BlockCacheManagerInner>,
}

pub struct BlockCacheManagerInner{
    block_device: Arc<dyn BlockDevice>,
}

impl BlockCacheManagerInner {
    pub fn new()->Self{
        BlockCacheManagerInner { block_device: Arc::new(BlockNone) }
    }
}

impl BlockCacheManager {
    pub fn new() -> Self {
        Self {
            ctrl: Mutex::new(BufLru::new()),
            bufs: array![_ => BufInner::new(); NBUF],
            inner:unsafe{UPSafeCell::new(BlockCacheManagerInner::new())},
        }
    }

    pub fn set_block_device(&self,block_device: Arc<dyn BlockDevice>){
        self.inner.exclusive_access().block_device=Arc::clone(&block_device);
    }

    /// Init the bcache.
    /// Should only be called once when the kernel inits itself.
    pub fn binit(&self) {
        let mut ctrl = self.ctrl.lock();
        let len = ctrl.inner.len();

        // init the head and tail of the lru list
        ctrl.head = &mut ctrl.inner[0];
        ctrl.tail = &mut ctrl.inner[len-1];

        // init prev and next field
        ctrl.inner[0].prev = ptr::null_mut();
        ctrl.inner[0].next = &mut ctrl.inner[1];
        ctrl.inner[len-1].prev = &mut ctrl.inner[len-2];
        ctrl.inner[len-1].next = ptr::null_mut();
        for i in 1..(len-1) {
            ctrl.inner[i].prev = &mut ctrl.inner[i-1];
            ctrl.inner[i].next = &mut ctrl.inner[i+1];
        }
        
        // init index
        ctrl.inner.iter_mut()
            .enumerate()
            .for_each(|(i, b)| b.index = i);
    }

    ///获取block device对应的buffer
    fn bget(&self,block_device: Arc<dyn BlockDevice>, dev: u32, blockno: u32) -> Buf<'_> {
        let ctrl = self.ctrl.lock();

        // find cached block
        match ctrl.find_cached(dev, blockno) {
            Some((index, rc_ptr)) => {
                // found
                drop(ctrl);
                Buf {
                    index,
                    dev,
                    block_device,
                    block_id: blockno,
                    rc_ptr,
                    data: Some(self.bufs[index].data.lock()),
                }
            }
            None => {
                // not cached
                // recycle the least recently used (LRU) unused buffer
                match ctrl.recycle(dev, blockno) {
                    Some((index, rc_ptr)) => {
                        self.bufs[index].valid.store(false, Ordering::Relaxed);
                        drop(ctrl);
                        return Buf {
                            index,
                            dev,
                            block_device,
                            block_id: blockno,
                            rc_ptr,
                            data: Some(self.bufs[index].data.lock()),
                        }
                    }
                    None => panic!("no usable buffer")
                }
            }
        }
    }

     /// Get the buf from the cache/disk(block device)
     pub fn bread<'a>(&'a self, dev: u32, block_id: u32) -> Buf<'a> {
        //info!("block id is {}",block_id);
        let inner=self.inner.exclusive_access();
        let mut b = self.bget(Arc::clone(&inner.block_device), dev, block_id);
        if !self.bufs[b.index].valid.load(Ordering::Relaxed) {
            //info!("not find block {} in cache!",block_id);
            inner.block_device.read_block(block_id as usize, b.data.as_mut().unwrap().0.as_mut());
            self.bufs[b.index].valid.store(true, Ordering::Relaxed);
        }
        drop(inner);
        b
    }

    /// Move an unlocked buf to the head of the most-recently-used list.
    fn brelse(&self, index: usize) {
        self.ctrl.lock().move_if_no_ref(index);
    }
}

//TODO！解决buffer data的互斥访问的问题
/// A wrapper of raw buf data.
pub struct Buf<'a>{
    index: usize,
    dev: u32,
    block_device: Arc<dyn BlockDevice>,
    block_id: u32,
    pub rc_ptr: *mut usize,     // pointer to its refcnt in BufCtrl
    /// Guaranteed to be Some during Buf's lifetime.
    /// Introduced to let the sleeplock guard drop before the whole struct.
    data:  Option<SleepLockGuard<'a, BufData>>,
}

impl<'a> Buf<'a> {
    pub fn read_blockno(&self) -> u32 {
        self.block_id
    }

    ///write data into block device
    pub fn bwrite(&mut self) {
        self.block_device.write_block(self.block_id as usize, self.data.as_ref().unwrap().0.as_ref());
    }

    /// Gives out a raw const pointer at the buf data. 
    pub fn raw_data(&self) -> *const BufData {
        let guard=self.data.as_ref().unwrap();
        guard.deref()
    }

    /// Gives out a raw mut pointer at the buf data. 
    pub fn raw_data_mut(&mut self) -> *mut BufData {
        let guard = self.data.as_mut().unwrap();
        guard.deref_mut()
    }

    /// Pin the buf.
    /// SAFETY: it should be definitly safe.
    ///     Because the current refcnt >= 1, so the rc_ptr is valid.
    pub unsafe fn pin(&self) {
        let rc = *self.rc_ptr;
        *self.rc_ptr = rc + 1;
        //info!("buf {} rc +1 = {}",self.block_id,*self.rc_ptr);
    }

    /// Unpin the buf.
    /// SAFETY: it should be called matching pin.
    pub unsafe fn unpin(&self) {
        //info!("buf {} rc = {}",self.block_id,*self.rc_ptr);
        let rc = *self.rc_ptr;
        if rc <= 1 {
            panic!("buf unpin not match");
        }
        *self.rc_ptr = rc - 1;
    }
}

impl<'a> Drop for Buf<'a> {
    fn drop(&mut self) {
        drop(self.data.take());
        BLOCK_CACHE_MANAGER.brelse(self.index);        
    }
}

struct BufLru {
    inner: [BufCtrl; NBUF],
    head: *mut BufCtrl,
    tail: *mut BufCtrl,
}

/// Raw pointers are automatically thread-unsafe.
/// See doc https://doc.rust-lang.org/nomicon/send-and-sync.html.
unsafe impl Send for BufLru {}

impl BufLru {
    const fn new() -> Self {
        Self {
            inner: array![_ => BufCtrl::new(); NBUF],
            head: ptr::null_mut(),
            tail: ptr::null_mut(),
        }
    }

    /// Find if the requested block is cached.
    /// Return its index and incr the refcnt if found.
    fn find_cached(&self, dev: u32, blockno: u32) -> Option<(usize, *mut usize)> {
        let mut b = self.head;
        while !b.is_null() {
            let bref = unsafe { b.as_mut().unwrap() };
            if bref.blockno == blockno {
                bref.refcnt += 1;
                return Some((bref.index, &mut bref.refcnt));
            }
            b = bref.next;
        }
        None
    }

    /// Recycle an unused buffer from the tail.
    /// Return its index if found.
    fn recycle(&self, dev: u32, blockno: u32) -> Option<(usize, *mut usize)> {
        debug!("[Xv6fs] BLOCK CACHE MANAGER: recycle unused buffer {}",blockno);
        let mut b = self.tail;
        while !b.is_null() {
            let bref = unsafe { b.as_mut().unwrap() };
            if bref.refcnt == 0 {
                bref.dev = dev;
                bref.blockno = blockno;
                bref.refcnt += 1;
                return Some((bref.index, &mut bref.refcnt));
            }
            b = bref.prev;
        }
        None
    }

    /// Move an entry to the head if no live ref.
    fn move_if_no_ref(&mut self, index: usize) {
        let b = &mut self.inner[index];
        b.refcnt -= 1;
        if b.refcnt == 0 && !ptr::eq(self.head, b) {
            // forward the tail if b is at the tail
            // b may be the only entry in the lru list
            if ptr::eq(self.tail, b) && !b.prev.is_null() {
                self.tail = b.prev;
            }
            
            // detach b
            unsafe {
                b.next.as_mut().map(|b_next| b_next.prev = b.prev);
                b.prev.as_mut().map(|b_prev| b_prev.next = b.next);
            }

            // attach b
            b.prev = ptr::null_mut();
            b.next = self.head;
            unsafe {
                self.head.as_mut().map(|old_head| old_head.prev = b);
            }
            self.head = b;
        }
    }
}

struct BufCtrl {
    dev: u32,
    blockno: u32,
    prev: *mut BufCtrl,
    next: *mut BufCtrl,
    refcnt: usize,
    index: usize,
}

impl BufCtrl {
    const fn new() -> Self {
        Self {
            dev: 0,
            blockno: 0,
            prev: ptr::null_mut(),
            next: ptr::null_mut(),
            refcnt: 0,
            index: 0,
        }
    }
}

struct BufInner {
    // valid is guarded by
    // the bcache spinlock and the relevant buf sleeplock
    // holding either of which can get access to them
    valid: AtomicBool,
    data: SleepLock<BufData>,
}

impl BufInner {
    fn new() -> Self {
        Self {
            valid: AtomicBool::new(false),
            data: SleepLock::new(BufData::new(),init_lock()),
        }
    }
}

/// Alignment of BufData should suffice for other structs
/// that might converts from this struct.
#[derive(Clone, Copy, Debug)]
#[repr(C, align(8))]
pub struct BufData([u8; BSIZE]);

impl  BufData {
    const fn new() -> Self {
        Self([0; BSIZE])
    }
}
