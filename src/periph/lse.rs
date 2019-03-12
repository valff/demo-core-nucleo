//! 32.768 kHz Low Speed External resonator.

use crate::reg;
use drone_core::periph;

periph::one! {
  /// Acquires LSE.
  pub macro periph_lse;

  /// LSE.
  pub struct LsePeriph;

  reg; periph::lse;

  RCC {
    BDCR {
      LSEBYP;
      LSEON;
      LSERDY;
    }
  }
}
