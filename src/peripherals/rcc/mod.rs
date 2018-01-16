//! Reset and clock control.

#[macro_use]
mod css;
#[macro_use]
mod pll;

pub use self::css::{Css, CssTokens};
pub use self::pll::{Pll, PllTokens};

use drone_core::peripheral::{PeripheralDevice, PeripheralTokens};
use drone_cortex_m::reg;
use drone_cortex_m::reg::prelude::*;

/// Creates a new `Rcc`.
#[macro_export]
macro_rules! peripheral_rcc {
  ($regs:ident, $thrd:ident) => {
    $crate::peripherals::rcc::Rcc::from_tokens(
      $crate::peripherals::rcc::RccTokens {
        flash_acr: $regs.flash_acr.into(),
        pwr_cr1: $regs.pwr_cr1.into(),
        rcc_apb1enr1: $regs.rcc_apb1enr1.into(),
        rcc_bdcr_lsebyp: $regs.rcc_bdcr.lsebyp,
        rcc_bdcr_lseon: $regs.rcc_bdcr.lseon,
        rcc_bdcr_lserdy: $regs.rcc_bdcr.lserdy,
        rcc_bdcr_rtcsel: $regs.rcc_bdcr.rtcsel,
        rcc_cfgr: $regs.rcc_cfgr.into(),
        rcc_cr_msipllen: $regs.rcc_cr.msipllen,
        rcc_cr_msirange: $regs.rcc_cr.msirange,
        rcc_cr_msirgsel: $regs.rcc_cr.msirgsel,
      }
    )
  }
}

/// Reset and clock control.
pub struct Rcc(RccTokens);

/// Reset and clock control tokens.
#[allow(missing_docs)]
pub struct RccTokens {
  pub flash_acr: reg::flash::Acr<Urt>,
  pub pwr_cr1: reg::pwr::Cr1<Urt>,
  pub rcc_apb1enr1: reg::rcc::Apb1Enr1<Urt>,
  pub rcc_bdcr_lsebyp: reg::rcc::bdcr::Lsebyp<Srt>,
  pub rcc_bdcr_lseon: reg::rcc::bdcr::Lseon<Srt>,
  pub rcc_bdcr_lserdy: reg::rcc::bdcr::Lserdy<Srt>,
  pub rcc_bdcr_rtcsel: reg::rcc::bdcr::Rtcsel<Srt>,
  pub rcc_cfgr: reg::rcc::Cfgr<Urt>,
  pub rcc_cr_msipllen: reg::rcc::cr::Msipllen<Srt>,
  pub rcc_cr_msirange: reg::rcc::cr::Msirange<Srt>,
  pub rcc_cr_msirgsel: reg::rcc::cr::Msirgsel<Srt>,
}

impl PeripheralTokens for RccTokens {
  // FIXME https://github.com/rust-lang/rust/issues/47385
  type InputTokens = Self;
}

impl PeripheralDevice for Rcc {
  type Tokens = RccTokens;

  #[inline(always)]
  fn from_tokens(tokens: RccTokens) -> Self {
    Rcc(tokens)
  }

  #[inline(always)]
  fn into_tokens(self) -> RccTokens {
    self.0
  }
}

impl Rcc {
  /// Configure maximum processor clock frequency of 80MHz.
  pub fn lse_init(
    mut self,
    pll: Pll,
  ) -> impl Future<Item = (Self, Pll), Error = !> {
    // Enable on-board LSE crystal.
    self.0.rcc_apb1enr1.modify(|r| r.set_pwren());
    self.0.pwr_cr1.modify(|r| r.set_dbp());
    self.0.rcc_bdcr_lseon.modify(|r| {
      self.0.rcc_bdcr_lseon.set(r);
      self.0.rcc_bdcr_lsebyp.clear(r);
      self.0.rcc_bdcr_rtcsel.write(r, 0b01);
    });
    while !self.0.rcc_bdcr_lserdy.read_bit() {}

    // Configure MSI to use hardware auto calibration with LSE.
    self.0.rcc_cr_msipllen.modify(|r| {
      self.0.rcc_cr_msipllen.set(r);
      self.0.rcc_cr_msirgsel.set(r);
      self.0.rcc_cr_msirange.write(r, 0b0111);
    });

    pll.init().and_then(move |pll| {
      // Setup flash to use at maximum performance.
      self
        .0
        .flash_acr
        .modify(|r| r.set_prften().set_icen().set_dcen().write_latency(2));
      self.0.rcc_cfgr.modify(|r| r.write_sw(0b11));
      Ok((self, pll))
    })
  }
}
