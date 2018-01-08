//! Interrupt-driven threads.

use drone_core::thread::thread_local;
use drone_cortex_m::vtable;

vtable! {
  /// The vector table.
  VectorTable;
  /// Thread bindings.
  ThreadIndex;
  /// Array of threads.
  THREADS;
  ThreadLocal;

  /// Non maskable interrupt.
  NMI;
  /// All classes of fault.
  HARD_FAULT;
  /// System tick timer.
  SYS_TICK;
  /// LED reactor.
  0: LED;
  /// Button reactor.
  1: BUTTON;
  /// RCC global interrupt.
  5: RCC;
  /// EXTI Line[15:10] interrupts.
  40: EXTI15_10;
}

thread_local! {
  /// An interrupt-driven thread.
  ThreadLocal;
  THREADS;
}
