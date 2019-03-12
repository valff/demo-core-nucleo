//! Interrupt-driven threads.

pub mod button;
pub mod led;
pub mod trunk;

use crate::sv::Sv;
use drone_core::thr;
use drone_cortex_m::vtable;
use drone_stm32_map::thr::*;

vtable! {
  /// The vector table.
  pub struct Vtable;
  /// Explicit handlers for the vector table.
  pub struct Handlers;
  /// Thread bindings.
  pub struct Thrs;
  /// Array of threads.
  static THREADS;
  extern struct Thr;

  /// Non maskable interrupt.
  pub NMI;
  /// All classes of fault.
  pub HARD_FAULT;
  /// System service call.
  fn SV_CALL;
  /// System tick timer.
  pub SYS_TICK;
  /// LED reactor.
  pub 0: LED;
  /// Button reactor.
  pub 1: BUTTON;
  /// EXTI Line[15:10] interrupts.
  pub 40: EXTI15_10;
}

thr! {
  /// An interrupt-driven thread.
  pub struct Thr;
  /// A thread-local storage.
  pub struct ThrLocal;
  extern struct Sv;
  extern static THREADS;
}
