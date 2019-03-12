//! Memory-mapped registers.

pub use drone_stm32_map::reg::*;

use drone_stm32_map::unsafe_stm32_reg_tokens;

unsafe_stm32_reg_tokens! {
  /// Register tokens for STM32L496.
  pub struct Regs;
}
