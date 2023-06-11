#![no_std]
use xv6fs::{file::{VFile,FileType},disk_inode::InodeType,xv6fs::Xv6FS, BlockDevice};
use vfscore::{VfsFile,SeekFrom,VfsFileSystem};
extern crate alloc;
use alloc::{boxed::Box,vec::Vec,string::String};
use axlog::info;

pub struct vfsFile{
    vfile:VFile,
}
impl VfsFile for vfsFile {
    fn open(&self, path: &str) -> Option<Box<dyn VfsFile>>{
        //info!("vfsfile: path is {}",path);
        let vfs_file=VFile::vfile_open(path,true, true).unwrap();
        Some(Box::new(vfsFile{vfile:vfs_file}))
    }
    fn mkdir(&self, folder_name: &str) -> Option<Box<dyn VfsFile>>{
        let vfs_file=self.vfile.vfile_create_under_dir(folder_name,InodeType::Directory);
        Some(Box::new(vfsFile{vfile:vfs_file}))
    }
    fn create(&self, file_name: &str) -> Option<Box<dyn VfsFile>>{
        let vfs_file=self.vfile.vfile_create_under_dir(file_name,InodeType::File);
        Some(Box::new(vfsFile{vfile:vfs_file}))
    }
    fn read_dir(&self) -> Vec<String>{
        self.vfile.vfile_readdir().unwrap()
    }
    fn read(&self, buf: &mut [u8]) -> usize{
        self.vfile.vfile_read(buf.as_mut_ptr() as usize, buf.len()).unwrap()
    }
    fn write(&self, data: &[u8]) -> usize{
        self.vfile.vfile_write(data.as_ptr() as usize, data.len()).unwrap()
    }
    fn seek(&self, seek: SeekFrom) -> usize{
        0
    }
    fn is_dir(&self) -> bool{
        self.vfile.vfile_is_dir()
    }
    fn is_file(&self) -> bool{
        self.vfile.vfile_is_file()
    }
    fn close(&self){

    }
    fn remove(&self, file_name: &str){
        //info!("path = {}",file_name);
        self.vfile.vfile_remove(file_name);
    }
    fn size(&self) -> usize{
        self.vfile.vfile_size()
    }
}

pub struct VXV6FS{
    pub fs:Xv6FS,
}

impl VfsFileSystem for VXV6FS{
    fn name(&self) -> &str{
        "xv6-log-fs"
    }
    fn root(&'static self) -> Box<dyn VfsFile>{
        let vfile=self.fs.get_root_vfile();
        Box::new(vfsFile{vfile})
    }
}

impl VXV6FS {
    pub fn new()->Self{
        Self { fs: Xv6FS::new() }
    }
}



