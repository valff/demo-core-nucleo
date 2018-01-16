use consts::{PLLCLK_FACTOR, PLL_INPUT_FACTOR, PLL_OUTPUT_FACTOR};
use drone_core::peripheral::{PeripheralDevice, PeripheralTokens};
use drone_cortex_m::reg;
use drone_cortex_m::reg::prelude::*;
use drone_cortex_m::thread::prelude::*;
use thread;

/// Creates a new `Pll`.
#[macro_export]
macro_rules! peripheral_rcc_pll {
  ($regs:ident, $thrd:ident) => {
    $crate::peripherals::rcc::Pll::from_tokens(
      $crate::peripherals::rcc::PllTokens {
        rcc: $thrd.rcc.into(),
        rcc_cicr_pllrdyc: $regs.rcc_cicr.pllrdyc,
        rcc_cier_pllrdyie: $regs.rcc_cier.pllrdyie,
        rcc_cifr_pllrdyf: $regs.rcc_cifr.pllrdyf,
        rcc_cr_pllon: $regs.rcc_cr.pllon,
        rcc_pllcfgr: $regs.rcc_pllcfgr.into(),
      }
    )
  }
}

/// PLL.
pub struct Pll(PllTokens<Frt>);

/// PLL tokens.
#[allow(missing_docs)]
pub struct PllTokens<Rt: RegTag> {
  pub rcc: thread::Rcc<Ltt>,
  pub rcc_cicr_pllrdyc: reg::rcc::cicr::Pllrdyc<Rt>,
  pub rcc_cier_pllrdyie: reg::rcc::cier::Pllrdyie<Srt>,
  pub rcc_cifr_pllrdyf: reg::rcc::cifr::Pllrdyf<Rt>,
  pub rcc_cr_pllon: reg::rcc::cr::Pllon<Rt>,
  pub rcc_pllcfgr: reg::rcc::Pllcfgr<Urt>,
}

impl From<PllTokens<Srt>> for PllTokens<Frt> {
  #[inline(always)]
  fn from(tokens: PllTokens<Srt>) -> Self {
    Self {
      rcc: tokens.rcc,
      rcc_cicr_pllrdyc: tokens.rcc_cicr_pllrdyc.into(),
      rcc_cier_pllrdyie: tokens.rcc_cier_pllrdyie,
      rcc_cifr_pllrdyf: tokens.rcc_cifr_pllrdyf.into(),
      rcc_cr_pllon: tokens.rcc_cr_pllon.into(),
      rcc_pllcfgr: tokens.rcc_pllcfgr,
    }
  }
}

impl PeripheralTokens for PllTokens<Frt> {
  type InputTokens = PllTokens<Srt>;
}

impl PeripheralDevice for Pll {
  type Tokens = PllTokens<Frt>;

  #[inline(always)]
  fn from_tokens(tokens: PllTokens<Srt>) -> Self {
    Pll(tokens.into())
  }

  #[inline(always)]
  fn into_tokens(self) -> PllTokens<Frt> {
    self.0
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
    let future = rcc.future(move || loop {
      if self.0.rcc_cifr_pllrdyf.read_bit() {
        self.0.rcc_cicr_pllrdyc.set_bit();
        break Ok(self);
      }
      yield;
    });
    pllon.set_bit();
    future
  }
}
