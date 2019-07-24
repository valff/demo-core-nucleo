//! Heap allocation.

use drone_core::heap;

heap! {
    /// The allocator struct.
    pub struct Heap;

    // extern fn ::drone_cortex_m::itm::trace_alloc;
    // extern fn ::drone_cortex_m::itm::trace_dealloc;
    // extern fn ::drone_cortex_m::itm::trace_grow_in_place;
    // extern fn ::drone_cortex_m::itm::trace_shrink_in_place;

    size = 0x40000;
    pools = [
        [0x4; 0x4000],
        [0x20; 0x800],
        [0x100; 0x100],
        [0x800; 0x20],
    ];
}
