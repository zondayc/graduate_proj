use clap::{App, Arg};
use xv6fs::bitmap::bfree;
use xv6fs::fs_const::BSIZE;
use xv6fs::inode::ICACHE;
use xv6fs::log::LOG_MANAGER;
use xv6fs::{BlockDevice,xv6fs::Xv6FileSystem,disk_inode::DiskInode,log::LogHeader,buffer_cache::BLOCK_CACHE_MANAGER};
use std::fs::{read_dir, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::mem::size_of;
use std::sync::Arc;
use std::sync::Mutex;

/// Use a block size of 1024 bytes
const BLOCK_SZ: usize = 1024;
const BLOCK_NUM: usize = 131072; //64*2048

struct BlockFile(Mutex<File>);

impl BlockDevice for BlockFile {
    /// Read a block from file
    fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        println!("read block {}",block_id);
        let mut file = self.0.lock().unwrap();
        file.seek(SeekFrom::Start((block_id * BLOCK_SZ) as u64))
            .expect("Error when seeking!");
        assert_eq!(file.read(buf).unwrap(), BLOCK_SZ, "Not a complete block!");
        //println!("read block {} buf {:?}",block_id,buf);
    }
    /// Write a block into file
    fn write_block(&self, block_id: usize, buf: &[u8]) {
        let mut file = self.0.lock().unwrap();
        file.seek(SeekFrom::Start((block_id * BLOCK_SZ) as u64))
            .expect("Error when seeking!");
        assert_eq!(file.write(buf).unwrap(), BLOCK_SZ, "Not a complete block!");
        //println!("write block {} with buf {:?}",block_id,buf);
    }
}

fn main(){

}

#[test]
fn xv6fs_test_create() -> std::io::Result<()> {
    let block_file = Arc::new(BlockFile(Mutex::new({
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("target/fs.img")?;
        f.set_len((BLOCK_NUM * BLOCK_SZ) as u64).unwrap();
        f
    })));
    println!("block size:{}, disk inode size:{}, log header size:{}",BSIZE,size_of::<DiskInode>(),size_of::<LogHeader>());
    let mut xfs=Xv6FileSystem::new();
    //xfs.create(block_file.clone());
    unsafe{xv6fs::init(block_file.clone(), 1);}
    let root_inode=xfs.get_root_inode();
    println!("root inode is {:?}",root_inode);
    let mut root_data=root_inode.lock();
    println!("root get locked");
    let dir_list=root_data.ls().unwrap();
    println!("{:?}",dir_list);
    drop(root_data);
    let path:&[u8]=b"/test\0\0\0";
    let path2:&[u8]=b"/test1\0\0";
    let path3:&[u8]=b"/test2\0\0";
    let path4:&[u8]=b"/testdir\0\0";
    let mut test_inode=ICACHE.create(&path, xv6fs::disk_inode::InodeType::File, 2, 1).unwrap();
    let mut test_inode2=ICACHE.create(&path2, xv6fs::disk_inode::InodeType::File, 2, 1).unwrap();
    let mut test_inode3=ICACHE.create(&path3, xv6fs::disk_inode::InodeType::File, 2, 1).unwrap();
    let mut test_inode4=ICACHE.create(&path4, xv6fs::disk_inode::InodeType::Directory, 2, 1).unwrap();
    let path5:&[u8]=b"/testdir/test7\0\0\0\0";
    let mut test_inode5=ICACHE.create(path5,xv6fs::disk_inode::InodeType::File, 2, 1).unwrap();
    let mut root_data=root_inode.lock();
    let dir_list=root_data.ls().unwrap();
    println!("{:?}",dir_list);
    drop(root_data);
    let mut test_data=test_inode4.lock();
    let dir_list=test_data.ls().unwrap();
    println!("{:?}",dir_list);
    LOG_MANAGER.commit_log();
    Ok(())
}

#[test]
fn xv6fs_log_delete() -> std::io::Result<()> {
    let block_file = Arc::new(BlockFile(Mutex::new({
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("target/fs.img")?;
        f.set_len((BLOCK_NUM * BLOCK_SZ) as u64).unwrap();
        f
    })));
    println!("block size:{}, disk inode size:{}, log header size:{}",BSIZE,size_of::<DiskInode>(),size_of::<LogHeader>());
    let mut xfs=Xv6FileSystem::new();
    //xfs.create(block_file.clone());
    unsafe{xv6fs::init(block_file.clone(), 1);}
    let root_inode=xfs.get_root_inode();
    //println!("root inode is {:?}",root_inode);
    let mut root_data=root_inode.lock();
    let dir_list=root_data.ls().unwrap();
    println!("{:?}",dir_list);
    drop(root_data);
    let mut buf = BLOCK_CACHE_MANAGER.bread(0, 2);
    let raw_lh = buf.raw_data_mut() as *mut LogHeader;
    println!("log header is {:?}",unsafe{raw_lh.as_ref().unwrap()});
    Ok(())
}

#[test]
fn xv6fs_test_write() -> std::io::Result<()> {
    let block_file = Arc::new(BlockFile(Mutex::new({
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("target/fs.img")?;
        f.set_len((BLOCK_NUM * BLOCK_SZ) as u64).unwrap();
        f
    })));
    let mut xfs=Xv6FileSystem::new();
    unsafe{xv6fs::init(block_file.clone(), 1);}
    let path:&[u8]=b"/test\0\0\0";
    let mut inode=ICACHE.create(path, xv6fs::disk_inode::InodeType::File, 2, 1).unwrap();
    let mut inode_data=inode.lock();
    let buf:&[u8]=b"1919810";
    inode_data.write(buf.as_ptr() as usize, 0, 7);
    drop(inode_data);
    drop(inode);
    LOG_MANAGER.commit_log();
    Ok(())
}

#[test]
fn xv6fs_ls_root() -> std::io::Result<()> {
    let block_file = Arc::new(BlockFile(Mutex::new({
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("target/fs.img")?;
        f.set_len((BLOCK_NUM * BLOCK_SZ) as u64).unwrap();
        f
    })));
    let mut xfs=Xv6FileSystem::new();
    unsafe{xv6fs::init(block_file.clone(), 1);}
    let root_inode=xfs.get_root_inode();
    //println!("root inode is {:?}",root_inode);
    let mut root_data=root_inode.lock();
    let dir_list=root_data.ls().unwrap();
    println!("{:?}",dir_list);
    drop(root_data);
    Ok(())
}

#[test]
fn xv6fs_test_read() -> std::io::Result<()> {
    let block_file = Arc::new(BlockFile(Mutex::new({
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("target/fs.img")?;
        f.set_len((BLOCK_NUM * BLOCK_SZ) as u64).unwrap();
        f
    })));
    println!("block size:{}, disk inode size:{}, log header size:{}",BSIZE,size_of::<DiskInode>(),size_of::<LogHeader>());
    let mut xfs=Xv6FileSystem::new();
    //xfs.create(block_file.clone());
    unsafe{xv6fs::init(block_file.clone(), 1);}
    let path:&[u8]=b"/test\0\0\0";
    let mut inode=ICACHE.create(path, xv6fs::disk_inode::InodeType::File, 2, 1).unwrap();
    let mut inode_data=inode.lock();
    let mut buf:[u8;10]=[0;10];
    inode_data.read(buf.as_mut_ptr() as usize, 0, 6);
    drop(inode_data);
    drop(inode);
    println!("buf is {:?}",String::from_utf8(buf.to_vec()).unwrap());
    Ok(())
    //获取root节点,ok
    //写入文件,ok
    //创建文件,ok
    //log的commit那里可能还有一些问题，这里估计还要修改,ok
    //还没有测试文件的读写，目录的读写测试也通过了，最后也写到了磁盘里面,ok
    //删除文件,这里用到的是unlink，这里需要自己实现咯，所以现在的任务应该只剩下实现若干系统调用&移植了
}

#[test]
fn xv6fs_test_bdealloc() -> std::io::Result<()> {
    let block_file = Arc::new(BlockFile(Mutex::new({
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("target/fs.img")?;
        f.set_len((BLOCK_NUM * BLOCK_SZ) as u64).unwrap();
        f
    })));
    let mut xfs=Xv6FileSystem::new();
    unsafe{xv6fs::init(block_file.clone(), 1);}
    bfree(0,47);
    LOG_MANAGER.commit_log();
    Ok(())
    //获取root节点,ok
    //写入文件,ok
    //创建文件,ok
    //log的commit那里可能还有一些问题，这里估计还要修改,ok
    //还没有测试文件的读写，目录的读写测试也通过了，最后也写到了磁盘里面,ok
    //删除文件,这里用到的是unlink，这里需要自己实现咯，所以现在的任务应该只剩下实现若干系统调用&移植了
}

#[test]
fn xv6fs_test_remove()->std::io::Result<()> {
    let block_file = Arc::new(BlockFile(Mutex::new({
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("target/fs.img")?;
        f.set_len((BLOCK_NUM * BLOCK_SZ) as u64).unwrap();
        f
    })));
    let mut xfs=Xv6FileSystem::new();
    unsafe{xv6fs::init(block_file.clone(), 1);}
    let path:&[u8]=b"/test\0\0\0";
    let rinode=ICACHE.get_root_dir();
    ICACHE.remove(path);
    let mut rdata=rinode.lock();
    //rdata.dir_unlink(path);
    let dir_list=rdata.ls().unwrap();
    println!("{:?}",dir_list);
    drop(rdata);
    drop(rinode);
    LOG_MANAGER.commit_log();
    //目录的nlink还没有处理
    Ok(())
}

#[test]
fn xv6fs_test_remove_dir()->std::io::Result<()> {
    let block_file = Arc::new(BlockFile(Mutex::new({
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("target/fs.img")?;
        f.set_len((BLOCK_NUM * BLOCK_SZ) as u64).unwrap();
        f
    })));
    let mut xfs=Xv6FileSystem::new();
    unsafe{xv6fs::init(block_file.clone(), 1);}
    let path:&[u8]=b"/testdir\0\0\0";
    let rinode=ICACHE.get_root_dir();
    ICACHE.remove(path);
    let mut rdata=rinode.lock();
    //rdata.dir_unlink(path);
    let dir_list=rdata.ls().unwrap();
    println!("{:?}",dir_list);
    drop(rdata);
    drop(rinode);
    LOG_MANAGER.commit_log();
    //目录的nlink还没有处理
    Ok(())
}