//! LED thread.

use crate::{consts::SYS_TICK_SEC, thr};
use core::sync::atomic::{AtomicBool, Ordering::*};
use drone_core::{awt_for, drv::timer::Timer};
use drone_cortex_m::{println, reg::prelude::*, thr::prelude::*};
use drone_stm32_drv::sys_tick::SysTick;
use drone_stm32_map::periph::gpio::pin::{
  GpioB14, GpioB7, GpioC7, GpioPinPeriph,
};
use futures::prelude::*;

const WIDTH: u32 = 6;
const STEP: u32 = (1 << WIDTH) * 2 / 3;

/// Determines the speed of LEDs blinking.
pub static FAST: AtomicBool = AtomicBool::new(false);

/// The thread input.
#[allow(missing_docs)]
pub struct Input {
  pub sys_tick: SysTick<thr::SysTick<Att>>,
  pub gpio_b14: GpioPinPeriph<GpioB14>,
  pub gpio_b7: GpioPinPeriph<GpioB7>,
  pub gpio_c7: GpioPinPeriph<GpioC7>,
}

/// The thread handler.
pub fn handler(input: Input) -> impl Future<Output = !> {
  let Input {
    mut sys_tick,
    gpio_b14,
    gpio_b7,
    gpio_c7,
  } = input;
  let mut counter = 0;
  asnc(static move || {
    let stream = sys_tick.interval_skip(SYS_TICK_SEC / 15_000);
    awt_for!(() in stream => {
      if counter & ((1 << 14) - 1) == 0 {
        println!("Counter: {}", counter);
      }
      let cycle = counter & ((1 << WIDTH) - 1);
      if cycle == 0 {
        gpio_b7.gpio_bsrr_br.store(|r| {
          gpio_b7.gpio_bsrr_br.set(r);
          gpio_b14.gpio_bsrr_br.set(r);
        });
        gpio_c7.gpio_bsrr_br.set_bit();
      }
      if cycle == pivot(counter, 0) {
        gpio_b14.gpio_bsrr_bs.set_bit();
      }
      if cycle == pivot(counter, STEP) {
        gpio_b7.gpio_bsrr_bs.set_bit();
      }
      if cycle == pivot(counter, STEP << 1) {
        gpio_c7.gpio_bsrr_bs.set_bit();
      }
      counter = if FAST.load(Relaxed) {
        counter.wrapping_add(0b100) & !0b011
      } else {
        counter.wrapping_add(0b001)
      };
    });
    unreachable!()
  })
}

fn pivot(counter: u32, offset: u32) -> u32 {
  let mut pivot = (counter >> WIDTH).wrapping_add(offset);
  if pivot & (1 << WIDTH + 1) != 0 {
    pivot = !pivot;
  }
  pivot & ((1 << WIDTH + 1) - 1)
}
