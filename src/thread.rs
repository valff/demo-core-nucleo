//! Interrupt-driven threads.

use drone::thread::thread_local;
use drone_cortex_m::vtable;

vtable! {
  //! The vector table.

  /// Non maskable interrupt.
  nmi;
  /// All classes of fault.
  hard_fault;
  /// System tick timer.
  sys_tick;
  /// RCC global interrupt.
  5: rcc;
  /// EXTI Line[15:10] interrupts.
  40: exti15_10;
}

thread_local! {
  //! An interrupt-driven thread.
}
