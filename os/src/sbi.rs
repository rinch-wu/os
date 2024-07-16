//! SBI call wrappers

/// use sbi call to putchar in console (qemu uart handler)
pub fn console_putchar(c: usize) {
    #[allow(deprecated)]
    sbi_rt::legacy::console_putchar(c);
}

/// use sbi call to getchar from console (qemu uart handler)
#[allow(unused)]
pub fn console_getchar() -> usize {
    #[allow(deprecated)]
    sbi_rt::legacy::console_getchar()
}

/// use sbi call to shutdown the kernel
pub fn shutdown(failure: bool) -> ! {
    use sbi_rt::{system_reset, NoReason, Shutdown, SystemFailure};
    if !failure {
        system_reset(Shutdown, NoReason);
    } else {
        system_reset(Shutdown, SystemFailure);
    }
    unreachable!()
}

/// set_timer
pub fn set_timer(timer: usize) {
    sbi_rt::set_timer(timer as _);
}

// use crate::board::CLOCK_FREQ;
// use crate::timer::get_time;

// const SBI_SET_TIMER: usize = 0;
// const TICKS_PER_SEC: usize = 100;

// pub fn set_next_trigger() {
//     set_timer(get_time() + CLOCK_FREQ / TICKS_PER_SEC);
// }
