use alloc::collections::vec_deque::VecDeque;

use crate::sync::{Condvar, UPIntrFreeCell};

pub struct NS16550a<const BASE_ADDR: usize> {
    inner: UPIntrFreeCell<NS16550aInner>,
    condvar: Condvar,
}

struct NS16550aInner {
    ns16550a: NS16550aRaw,
    read_buffer: VecDeque<usize>,
}

struct NS16550aRaw {
    base_addr: usize,
}
