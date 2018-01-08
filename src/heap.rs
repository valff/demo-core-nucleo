//! Heap allocation.
#![cfg_attr(feature = "clippy", allow(zero_ptr, unnecessary_mut_passed))]

use drone_core::heap;

heap! {
  /// The allocator struct.
  Heap;
  /// The global allocator.
  #[global_allocator]
  ALLOC;

  size = 0x40000;
  pools = [
    [0x4; 0x4000],
    [0x20; 0x800],
    [0x100; 0x100],
    [0x800; 0x20],
  ];
}
