#![feature(const_fn)]
#![feature(naked_functions)]
#![no_main]
#![no_std]
#![warn(clippy::pedantic)]

use demo_core_nucleo::{
  reg::Regs,
  sv::Sv,
  thr::{trunk, Handlers, Thr, Vtable},
  HEAP,
};
use drone_core::{heap::Allocator, mem, thr, token::Tokens};
use drone_cortex_m::{cpu, sv::sv_handler};

extern "C" {
  static mut BSS_START: usize;
  static BSS_END: usize;
  static mut DATA_START: usize;
  static DATA_END: usize;
  static DATA_CONST: usize;
  static mut HEAP_START: usize;
}

#[no_mangle]
pub static VTABLE: Vtable = Vtable::new(Handlers {
  reset,
  sv_call: sv_handler::<Sv>,
});

#[naked]
unsafe extern "C" fn reset() -> ! {
  mem::bss_init(&mut BSS_START, &BSS_END);
  mem::data_init(&mut DATA_START, &DATA_END, &DATA_CONST);
  HEAP.init(&mut HEAP_START);
  thr::init::<Thr>();
  trunk::handler(Regs::take());
  loop {
    cpu::wait_for_int();
  }
}
