#[no_mangle]
extern "C" fn main(_hartid: usize, device_tree_paddr: usize) {}

fn init_dt(dtb: usize) {}

fn walk_dt(fdt: Fdt) {}

fn virtio_probe(node: FdtNode) {}

fn virtio_gpt<T: Transport>(transport: T) {}
