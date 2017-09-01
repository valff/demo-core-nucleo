//! A SysTick exception is an exception the system timer generates when it
//! reaches zero.

use drone::exception::Exception;
use drone::reg::{Delegate, Reg, Sreg, ValuePointer};
use drone::reg::gpio::{self, BsrrBits, BsrrPin};

const WIDTH: u32 = 5;
const SPEED: u32 = 1;

static mut SYS_TICK: SysTick = SysTick {
  gpio_cbsrr: Reg::new(),
  counter: ((0b1 << (WIDTH * 2)) << SPEED) - 1,
};

/// The exception routine data.
pub struct SysTick {
  gpio_cbsrr: Sreg<gpio::Bsrr<gpio::port::B>>,
  counter: u32,
}

/// The exception configuration data.
pub struct SysTickConfig {
  /// Port B bit set/reset register.
  pub gpio_cbsrr: Sreg<gpio::Bsrr<gpio::port::B>>,
}

/// The exception handler.
pub extern "C" fn handler() {
  unsafe { SYS_TICK.run() }
}

impl Exception for SysTick {
  type Config = SysTickConfig;

  unsafe fn config(config: SysTickConfig) {
    let data = &mut SYS_TICK;
    data.gpio_cbsrr = config.gpio_cbsrr;
  }

  fn run(&mut self) {
    let gpio_cbsrr = self.gpio_cbsrr.ptr();
    let lightness = self.counter >> WIDTH >> SPEED;
    let position = self.counter & ((0b1 << WIDTH) - 1);
    if lightness == position {
      gpio_cbsrr.write(|reg| reg.output(BsrrPin::P7, false));
    } else if position == 0 {
      gpio_cbsrr.write(|reg| reg.output(BsrrPin::P7, true));
    }
    if self.counter == 0 {
      panic!();
    } else {
      self.counter -= 1;
    }
  }
}
