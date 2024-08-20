use super::BlockDevice;
use crate::drivers::VirtioHal;
use crate::sync::{Condvar, UPIntrFreeCell};
use alloc::collections::btree_map::BTreeMap;
use virtio_drivers::{VirtIOBlk, VirtIOHeader};

#[allow(unused)]
const VIRTIO0: usize = 0x10001000;

pub struct VirtIOBlock {
    virtio_blk: UPIntrFreeCell<VirtIOBlk<'static, VirtioHal>>,
    condvars: BTreeMap<u16, Condvar>,
}

impl BlockDevice for VirtIOBlock {
    fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        self.virtio_blk
            .exclusive_access()
            .read_block(block_id, buf)
            .expect("Error when reading VirtIOBlk");
    }
    fn write_block(&self, block_id: usize, buf: &[u8]) {
        self.virtio_blk
            .exclusive_access()
            .write_block(block_id, buf)
            .expect("Error when writing VirtIOBlk");
    }

    fn handle_irq(&self) {
        self.virtio_blk.exclusive_session(|blk| {
            while let Ok(token) = blk.pop_used() {
                self.condvars.get(&token).unwrap().wait_no_sched();
            }
        });
    }
}

impl VirtIOBlock {
    #[allow(unused)]
    pub fn new() -> Self {
        let virtio_blk =
            VirtIOBlk::<VirtioHal>::new(unsafe { &mut *(VIRTIO0 as *mut VirtIOHeader) }).unwrap();
        let mut condvars: BTreeMap<u16, Condvar> = BTreeMap::new();
        let channels = virtio_blk.virt_queue_size();
        for i in 0..channels {
            condvars.insert(0, Condvar::new());
        }
        Self {
            virtio_blk: unsafe { UPIntrFreeCell::new(virtio_blk) },
            condvars,
        }
    }
}
