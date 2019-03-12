//! 32.768 kHz Low Speed External resonator.

use crate::periph::lse::LsePeriph;
use drone_cortex_m::reg::prelude::*;

/// Acquires [`Lse`].
#[macro_export]
macro_rules! drv_lse {
  ($reg:ident) => {
    $crate::drv::lse::Lse::new(periph_lse!($reg))
  };
}

/// LSE driver.
pub struct Lse {
  periph: LsePeriph,
}

impl Lse {
  /// Creates a new [`Lse`].
  #[inline]
  pub fn new(periph: LsePeriph) -> Self {
    Self { periph }
  }

  /// Releases the peripheral.
  #[inline]
  pub fn free(self) -> LsePeriph {
    self.periph
  }

  /// Initializes LSE.
  pub fn init(&self) {
    // NOTE The crystal oscillator driving strength can be changed at runtime
    // using the LSEDRV[1:0] bits in the Backup domain control register
    // (RCC_BDCR) to obtain the best compromise between robustness and short
    // start-up time on one side and low-power-consumption on the other side.
    self.periph.rcc_bdcr_lseon.modify(|r| {
      self.periph.rcc_bdcr_lseon.set(r);
      self.periph.rcc_bdcr_lsebyp.clear(r);
    });
    while !self.periph.rcc_bdcr_lserdy.read_bit_band() {}
  }
}
