//! sleeplock

use core::ops::{Deref, DerefMut, Drop, Index};
use core::sync::atomic::{AtomicBool, fence, Ordering};
use core::cell::{Cell, UnsafeCell};
use axlog::info;
use spin::{Mutex,MutexGuard};

use crate::interface::INTERFACE_MANAGER;


pub struct SleepLock<T: ?Sized> {
    lock: Mutex<()>,
    index:usize,  //use for index the wait queue in the kernel, 
                //we use the wait queue in the kernel to guarantee the sleep lock
    data: UnsafeCell<T>,
    
}

unsafe impl<T: ?Sized + Sync> Sync for SleepLock<T> {}
// not needed
// unsafe impl<T: ?Sized + Sync> Send for SleepLock<T> {}

impl<T> SleepLock<T> {
    pub const fn new(data: T, index:usize) -> Self {
        Self {
            lock: Mutex::new(()),
            data: UnsafeCell::new(data),
            index,
        }
    }
    
}

pub fn init_lock()->usize{
        INTERFACE_MANAGER.interface.new_sleep_lock()
}

impl<T: ?Sized> SleepLock<T> {
    /// non-blocking, but might sleep if other p lock this sleeplock
    pub fn lock(&self) -> SleepLockGuard<T> {
        //let mut guard = self.lock.lock();
        //info!("xv6 sleep lock: lock!");
        //info!("1flag is {}",INTERFACE_MANAGER.interface.get_flag(self.index));
        INTERFACE_MANAGER.interface.sleep_cur_proc(self.index);
        //info!("2flag is {}",INTERFACE_MANAGER.interface.get_flag(self.index));
        //drop(guard);
        SleepLockGuard {
            lock: &self,
            data: unsafe { &mut *self.data.get() }
        }
    }

    /// Called by its guard when dropped
    pub fn unlock(&self) {
        //info!("unlock!");
        self.wake_up();//我感觉这里还是得搞个队列吧
    }

    fn wake_up(&self) {
        INTERFACE_MANAGER.interface.wake_up_next_proc(self.index);
    }
}


pub struct SleepLockGuard<'a, T: ?Sized + 'a> {
    lock: &'a SleepLock<T>,
    data: &'a mut T,
}

impl<'a, T: ?Sized> Deref for SleepLockGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &*self.data
    }
}

impl<'a, T: ?Sized> DerefMut for SleepLockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut *self.data
    }
}

impl<'a, T: ?Sized> Drop for SleepLockGuard<'a, T> {
    /// The dropping of the SpinLockGuard will call spinlock's release_lock(),
    /// through its reference to its original spinlock.
    fn drop(&mut self) {
        self.lock.unlock();
    }
}
