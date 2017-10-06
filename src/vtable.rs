//! The vector table to interrupt service routines.

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
}
