use alloc::collections::vec_deque::VecDeque;
use volatile::{ReadOnly, Volatile, WriteOnly};

use crate::{
    sync::{Condvar, UPIntrFreeCell},
    task::schedule,
};
use bitflags::*;

use super::CharDevice;

bitflags! {

    pub struct IER:u8{
        const RX_AVAILAVLE = 1<<0;
        const TX_EMPTY = 1<<0;
    }

    pub struct LSR:u8{
        const DATA_AVAILAVLE = 1<<0;
        const THR_EMPTY = 1<<0;
    }

    pub struct MCR:u8{
        const DATA_TERMINAL_READY = 1<<0;
        const REQUEST_TO_SEND = 1<<1;
        const AUX_OUTPUT1 = 1<<2;
        const AUX_OUTPUT2 = 1<<3;
    }
}

#[repr(C)]
#[allow(dead_code)]
struct ReadWithoutDLAB {
    pub rbr: ReadOnly<u8>,
    pub ier: Volatile<IER>,
    pub iir: ReadOnly<u8>,
    pub lcr: Volatile<u8>,
    pub mcr: Volatile<MCR>,
    pub lsr: ReadOnly<LSR>,
    _padding1: ReadOnly<u8>,
    _padding2: ReadOnly<u8>,
}

#[repr(C)]
#[allow(dead_code)]
struct WriteWithoutDLAB {
    pub thr: WriteOnly<u8>,
    pub ier: Volatile<IER>,
    _padding0: ReadOnly<u8>,
    pub lcr: Volatile<u8>,
    pub mcr: Volatile<MCR>,
    pub lsr: ReadOnly<LSR>,
    _padding1: ReadOnly<u16>,
}

pub struct NS16550a<const BASE_ADDR: usize> {
    inner: UPIntrFreeCell<NS16550aInner>,
    condvar: Condvar,
}

pub struct NS16550aInner {
    ns16550a: NS16550aRaw,
    read_buffer: VecDeque<u8>,
}

pub struct NS16550aRaw {
    base_addr: usize,
}

impl<const BASE_ADDR: usize> NS16550a<BASE_ADDR> {
    pub fn new() -> Self {
        let mut inner = NS16550aInner {
            ns16550a: NS16550aRaw::new(),
            read_buffer: VecDeque::new(),
        };
        inner.ns16550a.init();
        Self {
            inner: unsafe { UPIntrFreeCell::new(inner) },
            condvar: Condvar::new(),
        }
    }

    pub fn read_buffer_is_empty(&self) -> bool {
        self.inner
            .exclusive_session(|inner| inner.read_buffer.is_empty())
    }
}

impl<const BASE_ADDR: usize> CharDevice for NS16550a<BASE_ADDR> {
    fn init(&self) {
        todo!()
    }

    fn read(&self) -> u8 {
        loop {
            let mut inner = self.inner.exclusive_access();
            if let Some(ch) = inner.read_buffer.pop_front() {
                return ch;
            } else {
                let task_cx_ptr = self.condvar.wait_no_sched();
                drop(inner);
                schedule(task_cx_ptr);
            }
        }
    }

    fn write(&self, ch: u8) {
        let mut inner = self.inner.exclusive_access();
        inner.ns16550a.write(ch);
    }

    fn handle_irq(&self) {
        let mut count = 0;
        self.inner.exclusive_session(|inner| {
            while let Some(ch) = inner.ns16550a.read() {
                count += 1;
                inner.read_buffer.push_back(ch);
            }
        });
        if count > 0 {
            self.condvar.signal();
        }
    }
}

impl NS16550aRaw {
    pub fn new() -> Self {
        Self { base_addr: 0 }
    }

    pub fn init(&mut self) {
        let read_end = self.read_end();
        let mcr = MCR::DATA_TERMINAL_READY | MCR::REQUEST_TO_SEND | MCR::AUX_OUTPUT2;
        read_end.mcr.write(mcr);
    }

    pub fn read_end(&self) -> &mut ReadWithoutDLAB {
        unsafe { &mut *(self.base_addr as *mut ReadWithoutDLAB) }
    }
    pub fn write_end(&self) -> &mut WriteWithoutDLAB {
        unsafe { &mut *(self.base_addr as *mut WriteWithoutDLAB) }
    }

    pub fn read(&mut self) -> Option<u8> {
        let read_end = self.read_end();
        let lsr = read_end.lsr.read();
        if lsr.contains(LSR::DATA_AVAILAVLE) {
            Some(read_end.rbr.read())
        } else {
            None
        }
    }

    pub fn write(&mut self, ch: u8) {
        let write_end = self.write_end();
        loop {
            if write_end.lsr.read().contains(LSR::THR_EMPTY) {
                write_end.thr.write(ch);
                break;
            }
        }
    }
}
