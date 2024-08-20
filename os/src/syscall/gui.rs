use crate::{
    drivers::GPU_DEVICE,
    mm::{MapArea, MapPermission, MapType, PPNRange, PhysAddr},
    task::current_process,
};

const FB_VADDR: usize = 0x1000_0000;

pub fn sys_framebuffer() -> isize {
    let gpu = GPU_DEVICE.clone();
    let fb = gpu.get_framebuffer();
    let len = fb.len();
    let fb_ppn = PhysAddr::from(fb.as_ptr() as usize).floor();
    let fb_end_ppn = PhysAddr::from(fb.as_ptr() as usize + len).ceil();
    let current_process = current_process();
    let mut inner = current_process.inner_exclusive_access();

    let mem_set = &mut inner.memory_set;
    mem_set.push_noalloc(
        MapArea::new(
            (FB_VADDR as usize).into(),
            (FB_VADDR + len as usize).into(),
            MapType::Framed,
            MapPermission::R | MapPermission::W | MapPermission::U,
        ),
        PPNRange::new(fb_ppn, fb_end_ppn),
    );

    FB_VADDR as isize
}

pub fn sys_framebuffer_flush() -> isize {
    let gpu = GPU_DEVICE.clone();
    gpu.flush();
    0
}
