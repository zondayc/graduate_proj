//! sleeplock

use core::ops::{Deref, DerefMut, Drop};
use core::sync::atomic::{AtomicBool, fence, Ordering};
use core::cell::{Cell, UnsafeCell};
use spin::{Mutex,MutexGuard};

use crate::interface::INTERFACE_MANAGER;

pub struct SleepChannel(u8);

pub struct SleepLock<T: ?Sized> {
    lock: Mutex<()>,
    locked: Cell<bool>,
    chan: SleepChannel,
    name: &'static str,
    data: UnsafeCell<T>,
}

unsafe impl<T: ?Sized + Sync> Sync for SleepLock<T> {}
// not needed
// unsafe impl<T: ?Sized + Sync> Send for SleepLock<T> {}

impl<T> SleepLock<T> {
    pub const fn new(data: T, name: &'static str) -> Self {
        Self {
            lock: Mutex::new(()),
            locked: Cell::new(false),
            chan: SleepChannel(0),
            name,
            data: UnsafeCell::new(data),
        }
    }
}

impl<T: ?Sized> SleepLock<T> {
    /// non-blocking, but might sleep if other p lock this sleeplock
    pub fn lock(&self) -> SleepLockGuard<T> {
        let mut guard = self.lock.lock();
        while self.locked.get() {
            unsafe {
                INTERFACE_MANAGER.lock().interface.sleep_cur_proc(self.locked.as_ptr() as usize,guard);
            }
            guard = self.lock.lock();
        }
        self.locked.set(true);
        drop(guard);
        SleepLockGuard {
            lock: &self,
            data: unsafe { &mut *self.data.get() }
        }
    }

    /// Called by its guard when dropped
    pub fn unlock(&self) {
        let guard = self.lock.lock();
        self.locked.set(false);
        self.wake_up();
        drop(guard);
    }

    fn wake_up(&self) {
        unsafe{ 
            INTERFACE_MANAGER.lock().interface.wake_up_next_proc(self.locked.as_ptr() as usize);
        }
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
