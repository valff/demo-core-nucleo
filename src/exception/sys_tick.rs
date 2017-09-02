//! A SysTick exception is an exception the system timer generates when it
//! reaches zero.

use drone::exception::Exception;
use drone::reg::{Delegate, Reg, Sreg, ValuePointer};
use drone::reg::gpio::{self, BsrrBits, BsrrPin};

const WIDTH: u32 = 5;
const SPEED: u32 = 1;

static mut SYS_TICK: SysTick = SysTick {
  gpiob_cbsrr: Reg::new(),
  gpioc_cbsrr: Reg::new(),
  counter: ((0b1 << (WIDTH * 2)) << SPEED) - 1,
};

/// The exception routine data.
pub struct SysTick {
  gpiob_cbsrr: Sreg<gpio::Bsrr<gpio::port::B>>,
  gpioc_cbsrr: Sreg<gpio::Bsrr<gpio::port::C>>,
  counter: u32,
}

/// The exception configuration data.
pub struct SysTickConfig {
  /// Port B bit set/reset register.
  pub gpiob_cbsrr: Sreg<gpio::Bsrr<gpio::port::B>>,
  /// Port C bit set/reset register.
  pub gpioc_cbsrr: Sreg<gpio::Bsrr<gpio::port::C>>,
}

/// The exception handler.
pub extern "C" fn handler() {
  unsafe { SYS_TICK.run() }
}

impl Exception for SysTick {
  type Config = SysTickConfig;

  unsafe fn config(config: SysTickConfig) {
    let data = &mut SYS_TICK;
    data.gpiob_cbsrr = config.gpiob_cbsrr;
    data.gpioc_cbsrr = config.gpioc_cbsrr;
  }

  fn run(&mut self) {
    let gpiob_cbsrr = self.gpiob_cbsrr.ptr();
    let gpioc_cbsrr = self.gpioc_cbsrr.ptr();
    let lightness = self.counter >> WIDTH >> SPEED;
    let position = self.counter & ((0b1 << WIDTH) - 1);
    if lightness == position {
      gpiob_cbsrr.write(|reg| {
        reg.output(BsrrPin::P7, false).output(BsrrPin::P14, false)
      });
      gpioc_cbsrr.write(|reg| reg.output(BsrrPin::P7, false));
    } else if position == 0 {
      gpiob_cbsrr.write(|reg| {
        reg.output(BsrrPin::P7, true).output(BsrrPin::P14, true)
      });
      gpioc_cbsrr.write(|reg| reg.output(BsrrPin::P7, true));
    }
    if self.counter == 0 {
      panic!();
    } else {
      self.counter -= 1;
    }
  }
}
