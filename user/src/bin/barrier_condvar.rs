#![no_std]
#![no_main]

use core::cell::UnsafeCell;

#[macro_use]
extern crate user_lib;
extern crate alloc;

use lazy_static::*;
use user_lib::*;

const THREAD_NUM: usize = 3;

struct Barrier {
    mutex_id: usize,
    condvar_id: usize,
    count: UnsafeCell<usize>,
}

impl Barrier {
    pub fn new() -> Self {
        Self {
            mutex_id: mutex_create() as usize,
            condvar_id: condvar_create() as usize,
            count: UnsafeCell::new(0),
        }
    }

    pub fn block(&self) {
        mutex_lock(self.mutex_id);
        let count = self.count.get();
        unsafe {
            *count = *count + 1;
        }
        if unsafe { *count } == THREAD_NUM {
            condvar_signal(self.condvar_id);
        } else {
            condvar_wait(self.condvar_id, self.mutex_id);
            condvar_signal(self.condvar_id);
        }
        mutex_unlock(self.mutex_id);
    }
}

unsafe impl Sync for Barrier {}

lazy_static! {
    static ref BARRIER_AB: Barrier = Barrier::new();
    static ref BARRIER_BC: Barrier = Barrier::new();
}

fn thread_fn() -> i32 {
    for _ in 0..300 {
        print!("a");
    }
    BARRIER_AB.block();
    for _ in 0..300 {
        print!("b");
    }
    BARRIER_BC.block();
    for _ in 0..300 {
        print!("c");
    }
    exit(0)
}
