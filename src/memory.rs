use alloc_cortex_m::CortexMHeap;
use cortex_m_rt::heap_start;

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

pub fn init() {
    let start = heap_start() as usize;
    let size = 32768;
    unsafe { ALLOCATOR.init(start, size) }
}

#[alloc_error_handler]
fn alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Alloc failed! layout: {:?}", layout)
}