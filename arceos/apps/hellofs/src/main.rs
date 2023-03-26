#![no_std]
#![no_main]

use alloc::{string::String, vec::Vec};
use libax::info;

extern crate alloc;

fn test_directory() {
    // create a test directory
    libax::fs::create_dir("/totop\0".into()).expect("can't create directory");

    // whether the test directory exists
    // let finded: Vec<String> = libax::fs::read_dir("/".into())
    //     .expect("can't read directory")
    //     .into_iter()
    //     .filter(|x| x == "totop")
    //     .collect();
    // assert_eq!(finded.len(), 1);
    libax::fs::read_dir("/".into())
        .map(|x| {
            for file_name in x {
                libax::println!("{}", file_name);
            }
        })
        .expect("can't read root directory");
    // remove the directory
    libax::fs::remove_dir("/totop\0".into()).expect("can't remove directory");
    info!("end remove dir");
}

fn test_list_files() {
    // list files in the root directory
    libax::println!("{:=^30}", " file list ");
    libax::fs::read_dir("/".into())
        .map(|x| {
            for file_name in x {
                libax::println!("{}", file_name);
            }
        })
        .expect("can't read root directory");
    libax::println!("{:=^30}", " file list end ");
}

fn test_file() {
    // write a test file, if the file not exists, then create it
    libax::fs::write("/test\0".into(), b" Hello fs\n").expect("can't write to test file");
    libax::println!("end write");
    // read the file from the file
    let file_content = libax::fs::read("/test\0".into()).expect("can't read the test file");
    assert_eq!(file_content, b" Hello fs\n");

    // whether the file exists
    // let finded: Vec<String> = libax::fs::read_dir("/".into())
    //     .expect("can't read directory")
    //     .into_iter()
    //     .filter(|x| x == "test\0")
    //     .collect();
    // assert_eq!(finded.len(), 1);
    libax::fs::read_dir("/".into())
    .map(|x| {
        for file_name in x {
            libax::println!("{}", file_name);
        }
    })
    .expect("can't read root directory");
    // remove the file
    let res=libax::fs::read("/test\0".into()).unwrap();
    libax::println!("{:?}",core::str::from_utf8(&res.as_slice()));
    libax::fs::remove_file("/test\0".into()).expect("can't remove test file");
}

#[no_mangle]
fn main() {
    libax::println!("Hello, world!");

    test_list_files();
    test_directory();
    test_file();
}
