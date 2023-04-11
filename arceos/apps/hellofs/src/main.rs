#![no_std]
#![no_main]

use core::str;
use core::sync::atomic::AtomicU32;

use alloc::borrow::ToOwned;
use alloc::string::ToString;
use alloc::{string::String, vec::Vec};
use libax::info;
use libax::task;

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

fn test_sleep_lock(){
    libax::fs::test_sleep_lock();
}

fn test_link_unlink(){
    libax::fs::test_link_unlink();
}


fn test_concurrent_fs(){
    static  COUNTER:AtomicU32=AtomicU32::new(0);
    for i in 0..4{
        task::spawn(move||{
            let mut path="/test";
            let new_path=path.to_owned()+&i.to_string()+&"\0".to_owned();
            libax::fs::write(new_path.as_str().into(), b" Hello fs\n").expect("can't write to test file");
            libax::println!("end write");
            let file_content = libax::fs::read(new_path.as_str().into()).expect("can't read the test file");
            assert_eq!(file_content, b" Hello fs\n");
            task::yield_now();
            libax::fs::read_dir("/".into())
            .map(|x| {
                for file_name in x {
                    libax::println!("{}", file_name);
                }
            })
            .expect("can't read root directory");
            COUNTER.fetch_add(1, core::sync::atomic::Ordering::Acquire);
            libax::fs::remove_file(new_path.as_str().into());
        });
    }
    loop {
        if COUNTER.load(core::sync::atomic::Ordering::Acquire)==4{
            break;
        }
        task::yield_now();
    } 
    libax::fs::read_dir("/".into())
            .map(|x| {
                for file_name in x {
                    libax::println!("{}", file_name);
                }
            })
            .expect("can't read root directory");

    libax::println!("end test!");
}

fn test_huge_write(){
    libax::println!("{:=^30}", " file list ");
    libax::fs::read_dir("/".into())
            .map(|x| {
                for file_name in x {
                    libax::println!("{}", file_name);
                }
            })
            .expect("can't read root directory");
    // write a test file, if the file not exists, then create it
    let mut text=String::from("hello");
    let text2=String::from("fs");
    for _ in 0..40000{//bitmap分配这里有问题捏
        text=text.to_owned()+&text2.clone().to_owned();
    }
    libax::fs::write("/test\0".into(), text.as_bytes()).expect("can't write to test file");
    libax::println!("end write");
    // read the file from the file
    libax::fs::remove_file("/test\0".into()).expect("can't remove test file");
    libax::println!("{:=^30}", " file list ");
    libax::fs::read_dir("/".into())
            .map(|x| {
                for file_name in x {
                    libax::println!("{}", file_name);
                }
            })
            .expect("can't read root directory");
}
#[no_mangle]
fn main() {
    libax::println!("Hello, world!");

    //test_list_files();
    //test_directory();
    //test_file();
    
    //test_sleep_lock();
    //test_concurrent_fs();
    
    test_huge_write();
    
    //test_link_unlink();
}
