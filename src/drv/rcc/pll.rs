use consts::{PLLCLK_FACTOR, PLL_INPUT_FACTOR, PLL_OUTPUT_FACTOR};
use drone_core::drv::Resource;
use drone_plat::reg::prelude::*;
use drone_plat::thr::prelude::*;
use drone_plat::{fib, reg};
use futures::prelude::*;
use thr;

/// Creates a new `Pll`.
#[macro_export]
macro_rules! drv_rcc_pll {
  ($reg:ident, $thr:ident) => {
    $crate::drv::rcc::Pll::new($crate::drv::rcc::PllRes {
      rcc: $thr.rcc.into(),
      rcc_cicr_pllrdyc: $reg.rcc_cicr.pllrdyc,
      rcc_cier_pllrdyie: $reg.rcc_cier.pllrdyie,
      rcc_cifr_pllrdyf: $reg.rcc_cifr.pllrdyf,
      rcc_cr_pllon: $reg.rcc_cr.pllon,
      rcc_pllcfgr: $reg.rcc_pllcfgr.into(),
    })
  };
}

/// PLL driver.
#[derive(Driver)]
pub struct Pll(PllRes<Frt>);

/// PLL resource.
#[allow(missing_docs)]
pub struct PllRes<Rt: RegTag> {
  pub rcc: thr::Rcc<Ltt>,
  pub rcc_cicr_pllrdyc: reg::rcc::cicr::Pllrdyc<Rt>,
  pub rcc_cier_pllrdyie: reg::rcc::cier::Pllrdyie<Srt>,
  pub rcc_cifr_pllrdyf: reg::rcc::cifr::Pllrdyf<Rt>,
  pub rcc_cr_pllon: reg::rcc::cr::Pllon<Rt>,
  pub rcc_pllcfgr: reg::rcc::Pllcfgr<Urt>,
}

impl Resource for PllRes<Frt> {
  type Source = PllRes<Srt>;

  #[inline(always)]
  fn from_source(source: Self::Source) -> Self {
    Self {
      rcc: source.rcc,
      rcc_cicr_pllrdyc: source.rcc_cicr_pllrdyc.into(),
      rcc_cier_pllrdyie: source.rcc_cier_pllrdyie,
      rcc_cifr_pllrdyf: source.rcc_cifr_pllrdyf.into(),
      rcc_cr_pllon: source.rcc_cr_pllon.into(),
      rcc_pllcfgr: source.rcc_pllcfgr,
    }
  }
}

impl Pll {
  /// Configure PLL to use MSI on 80MHz.
  pub fn init(mut self) -> impl Future<Item = Self, Error = !> {
    self.0.rcc_pllcfgr.modify(|r| {
      r.write_pllsrc(0b01)
        .set_pllren()
        .write_pllr((PLLCLK_FACTOR >> 1) - 1)
        .write_pllm(PLL_INPUT_FACTOR - 1)
        .write_plln(PLL_OUTPUT_FACTOR)
    });
    self.0.rcc_cier_pllrdyie.set_bit();
    let rcc = self.0.rcc;
    let pllon = self.0.rcc_cr_pllon.fork();
    let future = fib::add_future(
      rcc,
      fib::new(move || loop {
        if self.0.rcc_cifr_pllrdyf.read_bit() {
          self.0.rcc_cicr_pllrdyc.set_bit();
          break Ok(self);
        }
        yield;
      }),
    );
    pllon.set_bit_band();
    future
  }
}
