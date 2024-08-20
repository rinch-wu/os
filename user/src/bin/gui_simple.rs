#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use core::slice::from_raw_parts_mut;

use user_lib::*;

pub const VIRTGPU_XRES: usize = 1280;
pub const VIRTGPU_YRES: usize = 800;

#[no_mangle]
pub fn main() -> i32 {
    let fb_ptr = framebuffer() as *mut u8;
    let fb = unsafe { from_raw_parts_mut(fb_ptr as *mut u8, VIRTGPU_XRES * VIRTGPU_YRES) };
    for y in 0..VIRTGPU_YRES {
        for x in 0..VIRTGPU_XRES {
            let idx = (y * VIRTGPU_XRES + x) * 4;
            fb[idx] = x as u8;
            fb[idx + 1] = y as u8;
            fb[idx + 2] = (x + y) as u8;
        }
    }
    framebuffer_flush();
    0
}
