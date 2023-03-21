use core::any::Any;

/// Trait for block devices
/// which reads and writes data in the unit of blocks
pub trait BlockDevice : Send + Sync + Any {
    fn read_block(&self, _block_id: usize, _buf: &mut [u8]);
    fn write_block(&self, _block_id: usize, _buf: &[u8]);
}

pub struct BlockNone;

impl BlockDevice for BlockNone{
    fn read_block(&self, _block_id: usize, _buf: &mut [u8]) {
        panic!("read from BlockNone!");
    }
    fn write_block(&self, _block_id: usize, _buf: &[u8]) {
        panic!("write from BlockNone!");
    }
}
