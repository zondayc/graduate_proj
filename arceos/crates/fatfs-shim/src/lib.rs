#![cfg_attr(not(test), no_std)]

#[macro_use]
extern crate alloc;
#[macro_use]
extern crate log;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::marker::PhantomData;

use fatfs::{
    Dir, File, FileSystem, LossyOemCpConverter, NullTimeProvider, Read, Seek, SeekFrom, Write,
};
use spin::Mutex;
use vfscore::{DiskOperation, VfsFile, VfsFileSystem};

pub struct Fat32FileSystem<T: DiskOperation>(
    fatfs::FileSystem<DiskCursor<T>, NullTimeProvider, LossyOemCpConverter>,
);

unsafe impl<T: DiskOperation> Send for Fat32FileSystem<T> {}

unsafe impl<T: DiskOperation> Sync for Fat32FileSystem<T> {}

impl<T: DiskOperation + 'static> VfsFileSystem for Fat32FileSystem<T> {
    fn root(&'static self) -> Box<dyn vfscore::VfsFile> {
        Inode::new_dir(self.0.root_dir())
    }

    fn name(&self) -> &str {
        "fat32"
    }
}

impl<T: DiskOperation> Fat32FileSystem<T> {
    pub fn new() -> Self {
        let cursor: DiskCursor<T> = DiskCursor {
            sector: 0,
            offset: 0,
            block_device: PhantomData,
        };
        Self(FileSystem::new(cursor, fatfs::FsOptions::new()).expect("open fs wrong"))
    }
}

pub enum Inode<T: DiskOperation + 'static> {
    /// File
    File(Arc<Mutex<File<'static, DiskCursor<T>, NullTimeProvider, LossyOemCpConverter>>>),
    /// Dir
    Dir(Arc<Mutex<Dir<'static, DiskCursor<T>, NullTimeProvider, LossyOemCpConverter>>>),
}

impl<T: DiskOperation> VfsFile for Inode<T> {
    fn open(&self, path: &str) -> Option<Box<dyn vfscore::VfsFile>> {
        match self {
            Inode::File(_) => None,
            Inode::Dir(dir) => {
                let dir = dir.lock();
                if let Ok(f) = dir.open_file(path) {
                    Some(Inode::new_file(f))
                } else if let Ok(d) = dir.open_dir(path) {
                    Some(Inode::new_dir(d))
                } else {
                    None
                }
            }
        }
    }

    fn read_dir(&self) -> Vec<String> {
        match self {
            Inode::File(_) => vec![],
            Inode::Dir(dir) => {
                let mut result = vec![];
                for f in dir.lock().iter() {
                    if let Ok(f) = f {
                        result.push(f.file_name());
                    }
                }

                result
            }
        }
    }

    fn read(&self, buf: &mut [u8]) -> usize {
        match self {
            Inode::File(file) => {
                let mut file = file.lock();
                let file_size = file.seek(fatfs::SeekFrom::End(0)).unwrap();
                file.seek(fatfs::SeekFrom::Start(0))
                    .expect("can't seek file");
                file.read_exact(buf).expect("can't read file exactly");
                file_size as usize
            }
            Inode::Dir(_) => 0,
        }
    }

    fn write(&self, data: &[u8]) -> usize {
        match self {
            Inode::File(file) => {
                let mut file = file.lock();
                file.write_all(data).expect("can't write file");
                data.len()
            }
            Inode::Dir(_) => 0,
        }
    }

    #[inline]
    fn close(&self) {
        // close file
    }

    fn mkdir(&self, folder_name: &str) -> Option<Box<dyn VfsFile>> {
        match self {
            Inode::File(_file) => None,
            Inode::Dir(dir) => {
                let file = dir.lock().create_dir(folder_name).map(Inode::new_dir);
                file.map(Some).unwrap_or(None)
            }
        }
    }

    fn create(&self, file_name: &str) -> Option<Box<dyn VfsFile>> {
        match self {
            Inode::File(_file) => None,
            Inode::Dir(dir) => {
                let file = dir.lock().create_file(file_name).map(Inode::new_file);
                file.map(Some).unwrap_or(None)
            }
        }
    }

    fn seek(&self, seek: vfscore::SeekFrom) -> usize {
        if let Inode::File(f) = self {
            match seek {
                vfscore::SeekFrom::Start(index) => {
                    f.lock().seek(SeekFrom::Start(index as u64)).unwrap() as _
                }
                vfscore::SeekFrom::Current(index) => {
                    f.lock().seek(SeekFrom::Current(index as i64)).unwrap() as _
                }
                vfscore::SeekFrom::End(index) => {
                    f.lock().seek(SeekFrom::End(index as i64)).unwrap() as _
                }
            }
        } else {
            0
        }
    }

    #[inline]
    fn is_dir(&self) -> bool {
        match self {
            Inode::File(_) => false,
            Inode::Dir(_) => true,
        }
    }

    #[inline]
    fn is_file(&self) -> bool {
        match self {
            Inode::File(_) => true,
            Inode::Dir(_) => false,
        }
    }

    #[inline]
    fn remove(&self, filename: &str) {
        if let Inode::Dir(dir) = self {
            dir.lock().remove(filename);
        }
    }

    #[inline]
    fn size(&self) -> usize {
        // self.seek(seek)
        if let Inode::File(file) = self {
            let mut file = file.lock();
            let current = file.seek(SeekFrom::Current(0)).expect("can't seek file");
            let file_size = file.seek(SeekFrom::End(0)).expect("can't seek file");
            file.seek(SeekFrom::Start(current))
                .expect("can't seek file");
            file_size as usize
        } else {
            0
        }
    }
}

impl<T: DiskOperation> Inode<T> {
    fn new_file(
        file: File<'static, DiskCursor<T>, NullTimeProvider, LossyOemCpConverter>,
    ) -> Box<dyn VfsFile> {
        Box::new(Self::File(Arc::new(Mutex::new(file))))
    }

    fn new_dir(
        dir: Dir<'static, DiskCursor<T>, NullTimeProvider, LossyOemCpConverter>,
    ) -> Box<dyn VfsFile> {
        Box::new(Self::Dir(Arc::new(Mutex::new(dir))))
    }
}

#[derive(Debug)]
pub enum DiskCursorIoError {
    UnexpectedEof,
    WriteZero,
}

impl fatfs::IoError for DiskCursorIoError {
    fn is_interrupted(&self) -> bool {
        false
    }

    fn new_unexpected_eof_error() -> Self {
        Self::UnexpectedEof
    }

    fn new_write_zero_error() -> Self {
        Self::WriteZero
    }
}

pub struct DiskCursor<T: DiskOperation> {
    sector: u64,
    offset: usize,
    block_device: PhantomData<T>,
}

unsafe impl<T: DiskOperation> Sync for DiskCursor<T> {}

unsafe impl<T: DiskOperation> Send for DiskCursor<T> {}

impl<T: DiskOperation> DiskCursor<T> {
    fn get_position(&self) -> usize {
        (self.sector * 0x200) as usize + self.offset
    }

    fn set_position(&mut self, position: usize) {
        self.sector = (position / 0x200) as u64;
        self.offset = position % 0x200;
    }

    fn move_cursor(&mut self, amount: usize) {
        self.set_position(self.get_position() + amount)
    }
}

impl<T: DiskOperation> fatfs::IoBase for DiskCursor<T> {
    type Error = DiskCursorIoError;
}

impl<T: DiskOperation> fatfs::Read for DiskCursor<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, DiskCursorIoError> {
        // 由于读取扇区内容还需要考虑跨 cluster，因此 read 函数只读取一个扇区
        // 防止读取较多数据时超出限制
        // 读取所有的数据的功能交给 read_exact 来实现

        // 如果 start 不是 0 或者 len 不是 512
        let read_size = if self.offset != 0 || buf.len() < 512 {
            // let mut data = [0u8; 512];
            let mut data = vec![0u8; 512];
            T::read_block(self.sector as usize, &mut data);
            // self.block_device

            let start = self.offset;
            let end = (self.offset + buf.len()).min(512);

            buf.copy_from_slice(&data[start..end]);
            end - start
        } else {
            T::read_block(self.sector as usize, &mut buf[0..512]);
            512
        };

        self.move_cursor(read_size);
        Ok(read_size)
    }
}

impl<T: DiskOperation> fatfs::Write for DiskCursor<T> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, DiskCursorIoError> {
        // 由于写入扇区还需要考虑申请 cluster，因此 write 函数只写入一个扇区
        // 防止写入较多数据时超出限制
        // 写入所有的数据的功能交给 write_all 来实现

        // 获取硬盘设备写入器（驱动？）
        // 如果 start 不是 0 或者 len 不是 512
        let write_size = if self.offset != 0 || buf.len() < 512 {
            let mut data = vec![0u8; 512];
            T::read_block(self.sector as usize, &mut data);

            let start = self.offset;
            let end = (self.offset + buf.len()).min(512);

            data[start..end].clone_from_slice(&buf);
            T::write_block(self.sector as usize, &mut data);

            end - start
        } else {
            T::write_block(self.sector as usize, &buf[0..512]);
            512
        };

        self.move_cursor(write_size);
        Ok(write_size)
    }

    fn flush(&mut self) -> Result<(), DiskCursorIoError> {
        Ok(())
    }
}

impl<T: DiskOperation> fatfs::Seek for DiskCursor<T> {
    fn seek(&mut self, pos: fatfs::SeekFrom) -> Result<u64, DiskCursorIoError> {
        match pos {
            fatfs::SeekFrom::Start(i) => {
                self.set_position(i as usize);
                Ok(i)
            }
            fatfs::SeekFrom::End(_) => {
                todo!("Seek from end")
            }
            fatfs::SeekFrom::Current(i) => {
                let new_pos = (self.get_position() as i64) + i;
                self.set_position(new_pos as usize);
                Ok(new_pos as u64)
            }
        }
    }
}
