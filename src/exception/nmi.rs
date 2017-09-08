//! A NonMaskable Interrupt (NMI) can be signalled by a peripheral or triggered
//! by software. This is the highest priority exception other than reset.

use drone::exception::Exception;
use drone_stm32::reg::{RccCicr, RccCifr};
use drone_stm32::reg::prelude::*;

static mut NMI: Nmi = Nmi {
  rcc_cifr: None,
  rcc_cicr: None,
};

/// The exception routine data.
pub struct Nmi {
  rcc_cifr: Option<RccCifr<Local>>,
  rcc_cicr: Option<RccCicr<Local>>,
}

/// The exception configuration data.
pub struct NmiConfig {
  /// Clock interrupt flag register.
  pub rcc_cifr: RccCifr<Local>,
  /// Clock interrupt clear register.
  pub rcc_cicr: RccCicr<Local>,
}

/// The exception handler.
pub extern "C" fn handler() {
  unsafe { NMI.run() }
}

impl Exception for Nmi {
  type Config = NmiConfig;

  unsafe fn config(config: NmiConfig) {
    let data = &mut NMI;
    data.rcc_cifr = Some(config.rcc_cifr);
    data.rcc_cicr = Some(config.rcc_cicr);
  }

  fn run(&mut self) {
    if let Some(ref rcc_cifr) = self.rcc_cifr {
      if rcc_cifr.read().lsecssf() {
        if let Some(ref mut rcc_cicr) = self.rcc_cicr {
          rcc_cicr.write_with(|reg| reg.set_lsecssc(true));
        }
        panic!("LSE clock failure");
      } else {
        panic!("Unknown NMI");
      }
    }
  }
}
