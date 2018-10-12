//! Heap allocation.

#![allow(clippy::zero_ptr, clippy::unnecessary_mut_passed)]

use core::alloc::Layout;
use drone_core::heap;
use drone_core::heap::Pool;

heap! {
  /// The allocator struct.
  pub struct Heap;
  extern fn alloc_hook;
  extern fn dealloc_hook;

  size = 0x40000;
  pools = [
    [0x4; 0x4000],
    [0x20; 0x800],
    [0x100; 0x100],
    [0x800; 0x20],
  ];
}

#[allow(unused_variables)]
#[inline(always)]
fn alloc_hook(layout: Layout, pool: &Pool) {
  // ::drone_plat::itm::instrument_alloc(layout, pool);
}

#[allow(unused_variables)]
#[inline(always)]
fn dealloc_hook(layout: Layout, pool: &Pool) {
  // ::drone_plat::itm::instrument_dealloc(layout, pool);
}
