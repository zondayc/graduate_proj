pub mod error;
pub mod filetype;
mod path;

pub use self::error::{Error, Result};
pub use alloc::{string::String, vec::Vec};
pub use path::Path;

use axfs::open;

/// Read a file from the given path.
pub fn read(path: Path) -> Result<Vec<u8>> {
    match open(path.as_path()) {
        Some(file) => {
            let file_size = file.size();
            let mut buffer = vec![0u8; file_size];
            file.read(&mut buffer);
            Ok(buffer)
        }
        None => Err(axerror::AxError::NotFound),
    }
}

// TODO: result to Result<ReadDirIterator>.
pub fn read_dir(path: Path) -> Result<Vec<String>> {
    match open(path.as_path()) {
        Some(file) => Ok(file.read_dir()),
        None => todo!(),
    }
}

pub fn read_to_string(path: Path) -> Result<String> {
    todo!()
}

pub fn write(path: Path, data: &[u8]) -> Result<()> {
    match open(path.as_path()) {
        Some(file) => {
            file.write(data);
            Ok(())
        }
        None => axfs::create(path.as_path()).map_or(Err(axerror::AxError::Unsupported), |f| {
            f.write(data);
            Ok(())
        }),
    }
}

pub fn remove_file(path: Path) -> Result<()> {
    axfs::remove(path.as_path());
    Ok(())
}

pub fn remove_dir(path: Path) -> Result<()> {
    // TODO: judge the directory whether it is empty or not
    axfs::remove(path.as_path());
    Ok(())
}

pub fn remove_dir_all(path: Path) -> Result<()> {
    axfs::remove(path.as_path());
    Ok(())
}

pub fn create_dir(path: Path) -> Result<()> {
    axfs::mkdir(path.as_path()).map_or(Err(axerror::AxError::NotFound), |_| Ok(()))
}
