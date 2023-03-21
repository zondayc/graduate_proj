#![no_std]
#![no_main]

use alloc::{string::String, vec::Vec};

extern crate alloc;

fn test_directory() {
    // create a test directory
    libax::fs::create_dir("/totop".into()).expect("can't create directory");

    // whether the test directory exists
    let finded: Vec<String> = libax::fs::read_dir("/".into())
        .expect("can't read directory")
        .into_iter()
        .filter(|x| x == "totop")
        .collect();
    assert_eq!(finded.len(), 1);

    // remove the directory
    libax::fs::remove_dir("/totop".into()).expect("can't remove directory");
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
    libax::fs::write("/test.txt".into(), b" Hello fs\n").expect("can't write to test file");

    // read the file from the file
    let file_content = libax::fs::read("/test.txt".into()).expect("can't read the test file");
    assert_eq!(file_content, b" Hello fs\n");

    // whether the file exists
    let finded: Vec<String> = libax::fs::read_dir("/".into())
        .expect("can't read directory")
        .into_iter()
        .filter(|x| x == "test.txt")
        .collect();
    assert_eq!(finded.len(), 1);

    // remove the file
    libax::fs::remove_file("/test.txt".into()).expect("can't remove test file");
}

#[no_mangle]
fn main() {
    libax::println!("Hello, world!");

    test_list_files();
    test_directory();
    test_file();
}
