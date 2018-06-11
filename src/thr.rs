//! Interrupt-driven threads.

use drone_core::thr;
use drone_plat::thr::int::*;
use drone_plat::vtable;
use sv::Sv;

vtable! {
  /// The vector table.
  pub struct Vtable;
  /// Explicit handlers for the vector table.
  pub struct Handlers;
  /// Thread bindings.
  pub struct ThrIdx;
  /// Array of threads.
  static THREADS;
  extern struct Thr;

  /// Non maskable interrupt.
  pub NMI;
  /// All classes of fault.
  pub HARD_FAULT;
  /// System tick timer.
  pub SYS_TICK;
  /// LED reactor.
  pub 0: LED;
  /// Button reactor.
  pub 1: BUTTON;
  /// RCC global interrupt.
  pub 5: RCC;
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
