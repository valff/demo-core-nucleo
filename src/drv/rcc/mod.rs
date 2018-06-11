//! Reset and clock control.

#[macro_use]
mod css;
#[macro_use]
mod pll;

pub use self::css::{Css, CssRes};
pub use self::pll::{Pll, PllRes};

use drone_plat::reg;
use drone_plat::reg::prelude::*;
use futures::prelude::*;

/// Creates a new `Rcc`.
#[macro_export]
macro_rules! drv_rcc {
  ($reg:ident) => {
    $crate::drv::rcc::Rcc::new($crate::drv::rcc::RccRes {
      flash_acr: $reg.flash_acr.into(),
      pwr_cr1: $reg.pwr_cr1.into(),
      rcc_apb1enr1: $reg.rcc_apb1enr1.into(),
      rcc_bdcr_lsebyp: $reg.rcc_bdcr.lsebyp,
      rcc_bdcr_lseon: $reg.rcc_bdcr.lseon,
      rcc_bdcr_lserdy: $reg.rcc_bdcr.lserdy,
      rcc_bdcr_rtcsel: $reg.rcc_bdcr.rtcsel,
      rcc_cfgr: $reg.rcc_cfgr.into(),
      rcc_cr_msipllen: $reg.rcc_cr.msipllen,
      rcc_cr_msirange: $reg.rcc_cr.msirange,
      rcc_cr_msirgsel: $reg.rcc_cr.msirgsel,
    })
  };
}

/// Reset and clock control driver.
#[derive(Driver)]
pub struct Rcc(RccRes);

/// Reset and clock control resource.
#[allow(missing_docs)]
#[derive(Resource)]
pub struct RccRes {
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
