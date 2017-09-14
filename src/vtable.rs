//! The vector table to interrupt service routines.

vtable! {
  #[doc = "Non maskable interrupt."]
  nmi,
  #[doc = "All classes of fault."]
  hard_fault,
  #[doc = "System tick timer."]
  sys_tick,
  #[doc = "RCC global interrupt."]
  irq5,
}

pub use self::irq5 as rcc;
