const VIRTIO5: usize = 0x1000_5000;
const VIRTIO6: usize = 0x1000_6000;

use core::any::Any;

use alloc::{collections::vec_deque::VecDeque, sync::Arc};
use lazy_static::*;
use virtio_drivers::{VirtIOHeader, VirtIOInput};

use crate::{
    sync::{Condvar, UPIntrFreeCell},
    task::schedule,
};

use super::VirtioHal;

struct VirtIOInputInner {
    virtio_input: VirtIOInput<'static, VirtioHal>,
    events: VecDeque<u64>,
}

struct VirtIOInputWrapper {
    inner: UPIntrFreeCell<VirtIOInputInner>,
    condvar: Condvar,
}

pub trait InputDevice: Send + Sync + Any {
    fn read_event(&self) -> u64;
    fn handle_irq(&self);
    fn is_empty(&self) -> bool;
}

lazy_static! {
    pub static ref KEYBOARD_DEVICE: Arc<dyn InputDevice> =
        Arc::new(VirtIOInputWrapper::new(VIRTIO5));
    pub static ref MOUSE_DEVICE: Arc<dyn InputDevice> = Arc::new(VirtIOInputWrapper::new(VIRTIO6));
}

impl VirtIOInputWrapper {
    pub fn new(addr: usize) -> Self {
        let inner = VirtIOInputInner {
            virtio_input: VirtIOInput::<VirtioHal>::new(unsafe {
                &mut *(addr as *mut VirtIOHeader)
            })
            .unwrap(),
            events: VecDeque::new(),
        };
        Self {
            inner: unsafe { UPIntrFreeCell::new(inner) },
            condvar: Condvar::new(),
        }
    }
}

impl InputDevice for VirtIOInputWrapper {
    fn read_event(&self) -> u64 {
        loop {
            let mut inner = self.inner.exclusive_access();
            if let Some(event) = inner.events.pop_front() {
                return event;
            } else {
                let task_cx_ptr = self.condvar.wait_no_sched();
                drop(inner);
                schedule(task_cx_ptr);
            }
        }
    }

    fn handle_irq(&self) {
        let mut count = 0;
        let mut result = 0;
        self.inner.exclusive_session(|inner| {
            inner.virtio_input.ack_interrupt();
            while let Some(event) = inner.virtio_input.pop_pending_event() {
                count += 1;
                result = (event.event_type as u64) << 48
                    | (event.code as u64) << 32
                    | (event.value as u64);
                inner.events.push_back(result);
            }
        });
        if count > 0 {
            self.condvar.signal();
        }
    }

    fn is_empty(&self) -> bool {
        self.inner.exclusive_access().events.is_empty()
    }
}
