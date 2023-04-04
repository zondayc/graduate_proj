use alloc::sync::Arc;
use lazy_static::*;
use core::{any::Any, sync::atomic::AtomicBool};
use spin::{Mutex, MutexGuard};

use crate::inode::Inode;

pub trait FsInterface:Send + Sync + Any {
    fn get_cur_dir_inode(&self)->Option<Inode>;
    fn sleep_cur_proc(&self,index:usize);
    fn wake_up_next_proc(&self,index:usize);
    fn new_sleep_lock(&self)->usize;
    fn get_flag(&self,index:usize)->bool;
}

pub struct InterfaceManager{
    pub interface:Arc<dyn FsInterface>,
}

pub struct InterfaceNone;

impl FsInterface for InterfaceNone {
    fn get_cur_dir_inode(&self)->Option<Inode> {
        None
    }

    fn sleep_cur_proc(&self,index:usize){
        panic!("not set interface!");
    }

    fn wake_up_next_proc(&self,index:usize){
        panic!("not set interface!");
    }
    fn new_sleep_lock(&self)->usize {
        0
    }
    fn get_flag(&self,index:usize)->bool {
        true
    }
}

impl InterfaceManager {
    
    pub fn new()->Self{
        Self { interface: Arc::new(InterfaceNone) }
    }

    pub fn set_interface(&mut self,interface:Arc<dyn FsInterface>){
        self.interface=Arc::clone(&interface);
    }
}

lazy_static!{
    pub static ref INTERFACE_MANAGER: Mutex<InterfaceManager>=Mutex::new(InterfaceManager::new());
}
