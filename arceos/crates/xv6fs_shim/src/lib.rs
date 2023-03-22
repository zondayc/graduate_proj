use xv6fs::{file::VFile,disk_inode::InodeType};
use vfscore::{VfsFile,SeekFrom};

pub struct vfsFile{
    vfile:VFile,
}
impl VfsFile for vfsFile {
    fn open(&self, path: &str) -> Option<Box<dyn VfsFile>>{
        let vfs_file=VFile::vfile_open(path,true, true).unwrap();
        Some(Box::new(vfsFile{vfile:vfs_file}))
    }
    fn mkdir(&self, folder_name: &str) -> Option<Box<dyn VfsFile>>{
        let vfs_file=self.vfile.vfile_create(folder_name,InodeType::Directory);
        Some(Box::new(vfsFile{vfile:vfs_file}))
    }
    fn create(&self, file_name: &str) -> Option<Box<dyn VfsFile>>{
        let vfs_file=self.vfile.vfile_create(file_name,InodeType::File);
        Some(Box::new(vfsFile{vfile:vfs_file}))
    }
    fn read_dir(&self) -> Vec<String>{
        self.vfile.vfile_readdir().unwrap()
    }
    fn read(&self, buf: &mut [u8]) -> usize{
        
        self.vfile.vfile_read(buf, len).unwrap()
    }
    fn write(&self, data: &[u8]) -> usize{

    }
    fn seek(&self, seek: SeekFrom) -> usize{

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
        self.vfile.vfile_remove(file_name);
    }
    fn size(&self) -> usize{
        self.vfile.vfile_size()
    }
}