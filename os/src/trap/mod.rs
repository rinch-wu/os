mod context;

pub use context::TrapContext;
use core::arch::global_asm;
use riscv::register::{stvec, utvec::TrapMode};

global_asm!(include_str!("trap.S"));

pub fn init() {
    extern "C" {
        fn __alltraps();
    }
    unsafe {
        stvec::write(__alltraps as usize, TrapMode::Direct);
    }
}
