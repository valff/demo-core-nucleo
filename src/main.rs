#![feature(const_fn)]
#![feature(naked_functions)]
#![no_main]
#![no_std]

extern crate demo_core_nucleo;
extern crate drone_core;
extern crate drone_stm32 as drone_plat;

use demo_core_nucleo::thr::{Handlers, Thr, Vtable};
use demo_core_nucleo::{trunk, HEAP};
use drone_core::heap::Allocator;
use drone_core::reg::RegTokens;
use drone_core::{mem, thr};
use drone_plat::cpu;
use drone_plat::reg::RegIdx;

extern "C" {
  static mut BSS_START: usize;
  static BSS_END: usize;
  static mut DATA_START: usize;
  static DATA_END: usize;
  static DATA_CONST: usize;
  static mut HEAP_START: usize;
}

#[no_mangle]
pub static VTABLE: Vtable = Vtable::new(Handlers { reset });

#[naked]
unsafe extern "C" fn reset() -> ! {
  mem::bss_init(&mut BSS_START, &BSS_END);
  mem::data_init(&mut DATA_START, &DATA_END, &DATA_CONST);
  HEAP.init(&mut HEAP_START);
  thr::init::<Thr>();
  start_trunk();
  loop {
    cpu::wait_for_int();
  }
}

#[inline(never)]
unsafe fn start_trunk() {
  trunk(RegIdx::new());
}
