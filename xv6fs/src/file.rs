use crate::fs_const::{ BSIZE, MAXOPBLOCKS };
use crate::inode::ICACHE;
use super::inode::Inode;
use super::stat::Stat;
use crate::log::LOG_MANAGER;

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
    pub fn read(
        &self, 
        addr: usize,
        len: usize
    ) -> Result<usize, &'static str> {
        let ret;
        if !self.readable() {
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
    pub fn write(
        &self, 
        addr: usize, 
        len: usize
    ) -> Result<usize, &'static str> {
        let ret; 
        if !self.writeable() {
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

    fn readable(&self) -> bool {
        self.readable
    }

    fn writeable(&self) -> bool {
        self.writeable
    }

    /// Get metadata about file f. 
    /// addr is a user virtual address, pointing to a struct stat. 
    pub fn stat(&self) -> Result<Stat, &'static str> {
        let mut stat: Stat = Stat::new();
        match self.ftype {
            FileType::File|FileType::Directory => {
                let inode = self.inode.as_ref().unwrap();
                
                #[cfg(feature = "debug")]
                println!("[Kernel] stat: inode index: {}, dev: {}, inum: {}", inode.index, inode.dev, inode.inum);

                let inode_guard = inode.lock();
                inode_guard.stat(&mut stat);
                drop(inode_guard);
                
                // println!(
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

    pub fn is_dir(&self)->bool{
        if self.ftype==FileType::Directory{
            return true;
        }
        false
    }

    pub fn is_file(&self)->bool{
        if self.ftype==FileType::File{
            return true;
        }
        false
    }

    pub fn open(path:&str,readable:bool,writeable:bool)->Option<Self>{
        let inode=ICACHE.create(path.as_bytes(),crate::disk_inode::InodeType::File, 2, 1).unwrap();
        Some(Self { ftype: FileType::File, readable, writeable, inode:Some(inode), offset:0, major:2})
    }

    pub fn mkdir(path:&str)->Option<Self>{
        let inode=ICACHE.create(path.as_bytes(),crate::disk_inode::InodeType::Directory, 2, 1).unwrap();
        Some(Self { ftype: FileType::Directory, readable:true, writeable:true, inode:Some(inode), offset:0, major:2})
    }

    pub fn readdir(&self)->Option<Vec<String>>{
        if self.ftype!=FileType::Directory{
            panic!("this is not a directory!");
        }
        let mut inode_data=self.inode.as_ref().unwrap().lock();
        inode_data.ls()
    }

    pub fn remove(path:&str){
        ICACHE.remove(path.as_bytes());
    }
}





