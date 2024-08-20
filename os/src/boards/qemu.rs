use crate::drivers::{CharDevice, IntrTargetPriority, NS16550a, BLOCK_DEVICE, PLIC, UART};

pub const CLOCK_FREQ: usize = 12500000;
pub const MEMORY_END: usize = 0x8800_0000;

pub const VIRT_PLIC: usize = 0xC00_0000;
pub const VIRT_UART: usize = 0x1000_0000;

pub const MMIO: &[(usize, usize)] = &[
    (0x0010_0000, 0x00_2000), // VIRT_TEST/RTC  in virt machine
    (0x1000_1000, 0x00_1000), // Virtio Block in virt machine
];

pub type BlockDeviceImpl = crate::drivers::block::VirtIOBlock;

pub type CharDeviceImpl = NS16550a<VIRT_UART>;

pub fn device_init() {}

pub fn irq_handler() {
    let mut plic = unsafe { PLIC::new(VIRT_PLIC) };
    let intr_src_id = plic.cliam(0, IntrTargetPriority::Supervisor);
    match intr_src_id {
        0 => BLOCK_DEVICE.handle_irq(),
        10 => UART.handle_irq(),
        _ => panic!("Unsupport IRQ {}", intr_src_id),
    }
    plic.complete(0, IntrTargetPriority::Supervisor, intr_src_id);
}
