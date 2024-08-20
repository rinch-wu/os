use core::{any::Any, slice::from_raw_parts_mut};

use alloc::{sync::Arc, vec::Vec};
use embedded_graphics::pixelcolor::Rgb888;
use lazy_static::*;
use tinybmp::Bmp;
use virtio_drivers::{VirtIOGpu, VirtIOHeader};

use crate::sync::UPIntrFreeCell;

use super::VirtioHal;

static BMP_DATA: &[u8] = include_bytes!("../../assert/mouse.bmp");

const VIRTIO7: usize = 0x1000_7000;

pub trait GpuDevice: Send + Sync + Any {
    fn update_cursor(&self);
    fn get_framebuffer(&self) -> &mut [u8];
    fn flush(&self);
}

pub struct VirtIOGpuWrapper {
    gpu: UPIntrFreeCell<VirtIOGpu<'static, VirtioHal>>,
    fb: &'static [u8],
}

lazy_static! {
    pub static ref GPU_DEVICE: Arc<dyn GpuDevice> = Arc::new(VirtIOGpuWrapper::new());
}

impl VirtIOGpuWrapper {
    pub fn new() -> Self {
        let mut virtio =
            VirtIOGpu::<VirtioHal>::new(unsafe { &mut *(VIRTIO7 as *mut VirtIOHeader) }).unwrap();
        let fbuffer = virtio.setup_framebuffer().unwrap();
        let len = fbuffer.len();
        let ptr = fbuffer.as_mut_ptr();
        let fb = unsafe { from_raw_parts_mut(ptr, len) };
        let bmp = Bmp::<Rgb888>::from_slice(BMP_DATA).unwrap();
        let raw = bmp.as_raw();
        let mut b = Vec::new();
        for i in raw.image_data().chunks(3) {
            let mut v = i.to_vec();
            b.append(&mut v);
            if i == [255, 255, 255] {
                b.push(0x0);
            } else {
                b.push(0xff);
            }
        }
        virtio.setup_cursor(b.as_slice(), 50, 50, 50, 50).unwrap();

        Self {
            gpu: unsafe { UPIntrFreeCell::new(virtio) },
            fb,
        }
    }
}

impl Default for VirtIOGpuWrapper {
    fn default() -> Self {
        Self::new()
    }
}

impl GpuDevice for VirtIOGpuWrapper {
    fn update_cursor(&self) {
        todo!()
    }

    fn get_framebuffer(&self) -> &mut [u8] {
        let ptr = self.fb.as_ptr() as *const _ as *mut u8;
        let len = self.fb.len();
        unsafe { from_raw_parts_mut(ptr, len) }
    }

    fn flush(&self) {
        self.gpu.exclusive_access().flush().unwrap();
    }
}
