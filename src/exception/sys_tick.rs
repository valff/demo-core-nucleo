//! A SysTick exception is an exception the system timer generates when it
//! reaches zero.

use drone::exception::Exception;
use drone_stm32::reg::{GpiobBsrr, GpiocBsrr};
use drone_stm32::reg::prelude::*;

const WIDTH: u32 = 5;
const SPEED: u32 = 1;

static mut SYS_TICK: SysTick = SysTick {
  gpiob_bsrr: None,
  gpioc_bsrr: None,
  counter: ((0b1 << (WIDTH * 2)) << SPEED) - 1,
};

/// The exception routine data.
pub struct SysTick {
  gpiob_bsrr: Option<GpiobBsrr<Local>>,
  gpioc_bsrr: Option<GpiocBsrr<Local>>,
  counter: u32,
}

/// The exception configuration data.
pub struct SysTickConfig {
  /// Port B bit set/reset register.
  pub gpiob_bsrr: GpiobBsrr<Local>,
  /// Port C bit set/reset register.
  pub gpioc_bsrr: GpiocBsrr<Local>,
}

/// The exception handler.
pub extern "C" fn handler() {
  unsafe { SYS_TICK.run() }
}

impl Exception for SysTick {
  type Config = SysTickConfig;

  unsafe fn config(config: SysTickConfig) {
    let data = &mut SYS_TICK;
    data.gpiob_bsrr = Some(config.gpiob_bsrr);
    data.gpioc_bsrr = Some(config.gpioc_bsrr);
  }

  fn run(&mut self) {
    if let Some(ref mut gpiob_bsrr) = self.gpiob_bsrr {
      if let Some(ref mut gpioc_bsrr) = self.gpioc_bsrr {
        let lightness = self.counter >> WIDTH >> SPEED;
        let position = self.counter & ((0b1 << WIDTH) - 1);
        if lightness == position {
          gpiob_bsrr.write_with(|reg| reg.set_br7(true).set_bs14(true));
          gpioc_bsrr.write_with(|reg| reg.set_bs7(true));
        } else if position == 0 {
          gpiob_bsrr.write_with(|reg| reg.set_bs7(true).set_br14(true));
          gpioc_bsrr.write_with(|reg| reg.set_br7(true));
        }
        if self.counter == 0 {
          panic!();
        } else {
          self.counter -= 1;
        }
      }
    }
  }
}
