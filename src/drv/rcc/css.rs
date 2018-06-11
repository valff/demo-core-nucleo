use drone_core::drv::Resource;
use drone_plat::reg::prelude::*;
use drone_plat::thr::prelude::*;
use drone_plat::{fib, reg};
use thr;

/// Creates a new `Css`.
#[macro_export]
macro_rules! drv_rcc_css {
  ($reg:ident, $thr:ident) => {
    $crate::drv::rcc::Css::new($crate::drv::rcc::CssRes {
      nmi: $thr.nmi.into(),
      rcc_bdcr_lsecsson: $reg.rcc_bdcr.lsecsson,
      rcc_cicr_lsecssc: $reg.rcc_cicr.lsecssc.into(),
      rcc_cier_lsecssie: $reg.rcc_cier.lsecssie,
      rcc_cifr_lsecssf: $reg.rcc_cifr.lsecssf.into(),
    })
  };
}

/// Clock security system driver.
#[derive(Driver)]
pub struct Css(CssRes<Frt>);

/// Clock security system resource.
#[allow(missing_docs)]
pub struct CssRes<Rt: RegTag> {
  pub nmi: thr::Nmi<Ltt>,
  pub rcc_bdcr_lsecsson: reg::rcc::bdcr::Lsecsson<Srt>,
  pub rcc_cicr_lsecssc: reg::rcc::cicr::Lsecssc<Rt>,
  pub rcc_cier_lsecssie: reg::rcc::cier::Lsecssie<Srt>,
  pub rcc_cifr_lsecssf: reg::rcc::cifr::Lsecssf<Rt>,
}

impl Resource for CssRes<Frt> {
  type Source = CssRes<Srt>;

  #[inline(always)]
  fn from_source(source: Self::Source) -> Self {
    Self {
      nmi: source.nmi,
      rcc_bdcr_lsecsson: source.rcc_bdcr_lsecsson,
      rcc_cicr_lsecssc: source.rcc_cicr_lsecssc.into(),
      rcc_cier_lsecssie: source.rcc_cier_lsecssie,
      rcc_cifr_lsecssf: source.rcc_cifr_lsecssf.into(),
    }
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
    fib::add(self.0.nmi, move || loop {
      if lsecssf.read_bit_band() {
        lsecssc.set_bit_band();
        break f();
      }
      yield;
    });
  }
}
