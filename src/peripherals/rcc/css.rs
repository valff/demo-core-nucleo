use drone_core::peripheral::{PeripheralDevice, PeripheralTokens};
use drone_cortex_m::reg;
use drone_cortex_m::reg::prelude::*;
use drone_cortex_m::thread::prelude::*;
use thread;

/// Creates a new `Css`.
#[macro_export]
macro_rules! peripheral_rcc_css {
  ($regs:ident, $thrd:ident) => {
    $crate::peripherals::rcc::Css::from_tokens(
      $crate::peripherals::rcc::CssTokens {
        nmi: $thrd.nmi.into(),
        rcc_bdcr_lsecsson: $regs.rcc_bdcr.lsecsson,
        rcc_cicr_lsecssc: $regs.rcc_cicr.lsecssc.into(),
        rcc_cier_lsecssie: $regs.rcc_cier.lsecssie,
        rcc_cifr_lsecssf: $regs.rcc_cifr.lsecssf.into(),
      }
    )
  }
}

/// Clock security system.
pub struct Css(CssTokens<Frt>);

/// Clock security system tokens.
#[allow(missing_docs)]
pub struct CssTokens<Rt: RegTag> {
  pub nmi: thread::Nmi<Ltt>,
  pub rcc_bdcr_lsecsson: reg::rcc::bdcr::Lsecsson<Srt>,
  pub rcc_cicr_lsecssc: reg::rcc::cicr::Lsecssc<Rt>,
  pub rcc_cier_lsecssie: reg::rcc::cier::Lsecssie<Srt>,
  pub rcc_cifr_lsecssf: reg::rcc::cifr::Lsecssf<Rt>,
}

impl From<CssTokens<Srt>> for CssTokens<Frt> {
  #[inline(always)]
  fn from(tokens: CssTokens<Srt>) -> Self {
    Self {
      nmi: tokens.nmi,
      rcc_bdcr_lsecsson: tokens.rcc_bdcr_lsecsson,
      rcc_cicr_lsecssc: tokens.rcc_cicr_lsecssc.into(),
      rcc_cier_lsecssie: tokens.rcc_cier_lsecssie,
      rcc_cifr_lsecssf: tokens.rcc_cifr_lsecssf.into(),
    }
  }
}

impl PeripheralTokens for CssTokens<Frt> {
  type InputTokens = CssTokens<Srt>;
}

impl PeripheralDevice for Css {
  type Tokens = CssTokens<Frt>;

  #[inline(always)]
  fn from_tokens(tokens: CssTokens<Srt>) -> Self {
    Css(tokens.into())
  }

  #[inline(always)]
  fn into_tokens(self) -> CssTokens<Frt> {
    self.0
  }
}

impl Css {
  /// Initialized CSS on LSE.
  pub fn lse_init(&self) {
    self.0.rcc_cier_lsecssie.set_bit();
    self.0.rcc_bdcr_lsecsson.set_bit();
  }

  /// Calls `f` on LSE failure.
  pub fn on_lse<F>(&mut self, f: F)
  where
    F: FnOnce() + Send + 'static,
  {
    let lsecssf = self.0.rcc_cifr_lsecssf.fork();
    let lsecssc = self.0.rcc_cicr_lsecssc.fork();
    self.0.nmi.routine(move || loop {
      if lsecssf.read_bit() {
        lsecssc.set_bit();
        break f();
      }
      yield;
    });
  }
}
