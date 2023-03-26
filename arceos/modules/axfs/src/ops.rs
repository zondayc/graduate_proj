use alloc::boxed::Box;
use vfscore::VfsFile;

use crate::mount::MOUNTEDFS;

/// open file with given path
pub fn open(path: &str) -> Option<Box<dyn VfsFile>> {
    let fs = MOUNTEDFS.get_matched_fs(path)?;
    //let path = &path[fs.path().len()..];
    let path = &path[0..];
    info!("open: path is {}",path);
    if path.len() > 1 {
        fs.fs().root().open(path)
    } else {
        Some(fs.fs().root())
    }
}

/// create a new file by given path
pub fn create(path: &str) -> Option<Box<dyn VfsFile>> {
    let fs = MOUNTEDFS.get_matched_fs(path)?;
    let path = &path[fs.path().len()..];
    if path.len() > 0 {
        fs.fs().root().create(path)
    } else {
        None
    }
}

/// create a new directory by given path
pub fn mkdir(path: &str) -> Option<Box<dyn VfsFile>> {
    let fs = MOUNTEDFS.get_matched_fs(path)?;
    let path = &path[fs.path().len()..];
    info!("mkdir: path is {}",path);
    if path.len() > 0 {
        fs.fs().root().mkdir(path)
    } else {
        None
    }
}

/// remove a file or directory
pub fn remove(path: &str) -> Option<()> {
    let fs = MOUNTEDFS.get_matched_fs(path)?;
    info!("path is {}",path);
    //let path = &path[fs.path().len()..];
    info!("path is {}",path);
    Some(fs.fs().root().remove(path))
}
