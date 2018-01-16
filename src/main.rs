#![feature(const_fn)]
#![no_main]
#![no_std]

extern crate demo_core_nucleo;
extern crate drone_core;
extern crate drone_cortex_m;

use demo_core_nucleo::{origin, ALLOC};
use demo_core_nucleo::thread::{ThreadLocal, VectorTable};
use drone_core::{mem, thread};
use drone_core::heap::Allocator;
use drone_core::origin::OriginToken;
use drone_cortex_m::{itm, mcu};

extern "C" {
  static mut BSS_START: usize;
  static BSS_END: usize;
  static mut DATA_START: usize;
  static DATA_END: usize;
  static DATA_CONST: usize;
  static mut HEAP_START: usize;
}

#[no_mangle]
pub static VECTOR_TABLE: VectorTable = VectorTable::new(reset);

unsafe extern "C" fn reset() -> ! {
  mem::bss_init(&mut BSS_START, &BSS_END);
  mem::data_init(&mut DATA_START, &DATA_END, &DATA_CONST);
  ALLOC.init(&mut HEAP_START);
  thread::init::<ThreadLocal>();
  itm::init();
  origin(OriginToken::new());
  loop {
    mcu::wait_for_interrupt();
  }
}
