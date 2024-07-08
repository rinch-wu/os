#![no_std]
#![feature(panic_info_message)]
#![no_main]

#[macro_use]
mod console;

global_asm!(include_str!("entry.asm"));
use core::arch::global_asm;
mod lang_items;
mod sbi;

#[no_mangle]
pub fn rust_main() -> ! {
    clear_bss();
    println!("Hello, world!");
    panic!("Shutdown machine!");
}

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| unsafe { (a as *mut u8).write_volatile(0) })
}
