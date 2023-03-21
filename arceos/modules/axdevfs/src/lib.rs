#![no_std]

pub trait Device {
    fn open(&self, sign: usize);
    fn close(&self);
    fn read(&self, data: &mut [u8]);
    fn write(&self, data: &[u8]);
    fn ioctl(&self, cmd: usize, arg: &mut [u8]); // ioctl read/write
    fn devctl(&self, cmd: usize, arg: &mut [u8]); // devctl read/write
}
