//! A NonMaskable Interrupt (NMI) can be signalled by a peripheral or triggered
//! by software. This is the highest priority exception other than reset.

use drone::exception::Exception;
use drone::reg::{Delegate, Reg, Sreg, ValuePointer};
use drone::reg::rcc::{self, CicrBits, CifrBits};

static mut NMI: Nmi = Nmi {
  rcc_cifr: Reg::new(),
  rcc_cicr: Reg::new(),
};

/// The exception routine data.
pub struct Nmi {
  rcc_cifr: Sreg<rcc::Cifr>,
  rcc_cicr: Sreg<rcc::Cicr>,
}

/// The exception configuration data.
pub struct NmiConfig {
  /// Clock interrupt flag register.
  pub rcc_cifr: Sreg<rcc::Cifr>,
  /// Clock interrupt clear register.
  pub rcc_cicr: Sreg<rcc::Cicr>,
}

/// The exception handler.
pub extern "C" fn handler() {
  unsafe { NMI.run() }
}

impl Exception for Nmi {
  type Config = NmiConfig;

  unsafe fn config(config: NmiConfig) {
    let data = &mut NMI;
    data.rcc_cifr = config.rcc_cifr;
    data.rcc_cicr = config.rcc_cicr;
  }

  fn run(&mut self) {
    let rcc_cifr = self.rcc_cifr.ptr();
    if rcc_cifr.read().lse_css() {
      let rcc_cicr = self.rcc_cicr.ptr();
      rcc_cicr.modify(|reg| reg.lse_css_clear());
      panic!("LSE clock failure");
    } else {
      panic!("Unknown NMI");
    }
  }
}
