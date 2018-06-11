//! The entry point.

use consts::SYS_TICK_SEC;
use core::sync::atomic::Ordering::*;
use core::sync::atomic::{AtomicBool, AtomicU32};
use drone_core::drv::Driver;
use drone_plat::drv::exti::ExtiLn13;
use drone_plat::drv::gpio::{GpioB14, GpioB7, GpioC13, GpioC7, GpioPin};
use drone_plat::drv::timer::SysTick;
use drone_plat::fib;
use drone_plat::reg::prelude::*;
use drone_plat::reg::{self, RegIdx};
use drone_plat::thr::prelude::*;
use futures::prelude::*;
use thr::{self, ThrIdx};

static FAST: AtomicBool = AtomicBool::new(false);
static DEBOUNCE: AtomicU32 = AtomicU32::new(0);

/// The entry point.
pub fn trunk(reg: RegIdx) {
  let thr: ThrIdx = drv_thr!(reg).init(|scb_ccr| scb_ccr.set_div_0_trp());
  let itm = drv_itm!(reg);
  let rcc = drv_rcc!(reg);
  let rcc_pll = drv_rcc_pll!(reg, thr);
  let mut rcc_css = drv_rcc_css!(reg, thr);
  let sys_tick = drv_sys_tick!(reg, thr);
  let gpio_b14 = drv_gpio_b14!(reg);
  let gpio_b7 = drv_gpio_b7!(reg);
  let gpio_c13 = drv_gpio_c13!(reg);
  let gpio_c7 = drv_gpio_c7!(reg);
  let exti13 = drv_exti_ln_13!(reg, thr);

  let RegIdx {
    rcc_ahb2enr,
    rcc_apb2enr,
    ..
  } = reg;

  itm.init();

  // Panic on Hard Fault.
  fib::add_fn(thr.hard_fault, || panic!("Hard Fault"));

  // Panic on LSE failure.
  rcc_css.lse_init();
  rcc_css.on_lse(|| panic!("LSE clock failure"));

  thr.rcc.enable_int();
  let setup = rcc.lse_init(rcc_pll).and_then(move |_| {
    thr.led.enable_batch(|r| {
      thr.led.enable(r);
      thr.button.enable(r);
    });
    thr.exti15_10.enable_int();
    // Configure push-button and LED pins.
    peripheral_init(
      &gpio_b14,
      &gpio_b7,
      &gpio_c13,
      &gpio_c7,
      &exti13,
      &mut rcc_ahb2enr.into(),
      &mut rcc_apb2enr.into(),
    );
    // Setup push-button routine.
    thr.button.exec(button_future(exti13, thr.sys_tick.into()));
    // Setup LED blink routine.
    thr
      .led
      .exec(led_future(sys_tick, gpio_b14, gpio_b7, gpio_c7));
    Ok(())
  });
  setup.trunk_wait().ok();

  // Do not return to the reset handler from interrupts.
  reg.scb_scr.modify(|r| r.set_sleeponexit());
}

#[cfg_attr(feature = "clippy", allow(too_many_arguments))]
fn peripheral_init(
  gpio_b14: &GpioB14<Srt>,
  gpio_b7: &GpioB7<Srt>,
  gpio_c13: &GpioC13<Srt>,
  gpio_c7: &GpioC7<Srt>,
  exti13: &ExtiLn13<thr::Exti1510<Ltt>>,
  rcc_ahb2enr: &mut reg::rcc::Ahb2Enr<Urt>,
  rcc_apb2enr: &mut reg::rcc::Apb2Enr<Urt>,
) {
  rcc_apb2enr.modify(|r| r.set_syscfgen());
  rcc_ahb2enr.modify(|r| r.set_gpioben().set_gpiocen());
  exti13.exticr_exti().write_bits(0b0010);
  exti13.imr_mr().set_bit();
  exti13.rtsr_rt().set_bit();
  gpio_b7.moder().modify(|r| {
    gpio_b7.moder().write(r, 0b01);
    gpio_b14.moder().write(r, 0b01);
  });
  gpio_c7.moder().modify(|r| {
    gpio_c7.moder().write(r, 0b01);
    gpio_c13.moder().write(r, 0b00);
  });
  gpio_b7.otyper().modify(|r| {
    gpio_b7.otyper().clear(r);
    gpio_b14.otyper().clear(r);
  });
  gpio_c7.otyper().clear_bit();
  gpio_b7.ospeedr().modify(|r| {
    gpio_b7.ospeedr().write(r, 0b11);
    gpio_b14.ospeedr().write(r, 0b11);
  });
  gpio_c7.ospeedr().write_bits(0b11);
  gpio_c13.pupdr().write_bits(0b10);
}

fn led_future(
  mut sys_tick: SysTick<thr::SysTick<Ltt>>,
  gpio_b14: GpioB14<Srt>,
  gpio_b7: GpioB7<Srt>,
  gpio_c7: GpioC7<Srt>,
) -> impl Future<Item = (), Error = !> {
  const WIDTH: u32 = 6;
  const STEP: u32 = (1 << WIDTH) * 2 / 3;
  let mut counter = 0;

  fn pivot(counter: u32, offset: u32) -> u32 {
    let mut pivot = (counter >> WIDTH).wrapping_add(offset);
    if pivot & (1 << WIDTH + 1) != 0 {
      pivot = !pivot;
    }
    pivot & ((1 << WIDTH + 1) - 1)
  }

  // Setup SysTick timer.
  let ctrl = sys_tick.ctrl().default().val();
  let stream = sys_tick.interval_skip(SYS_TICK_SEC / 15_000, ctrl);
  async(move || {
    await_for!(() in stream => {
      if counter & ((1 << 14) - 1) == 0 {
        println!("Counter: {}", counter);
      }
      let cycle = counter & ((1 << WIDTH) - 1);
      if cycle == 0 {
        gpio_b7.bsrr_br().store(|r| {
          gpio_b7.bsrr_br().set(r);
          gpio_b14.bsrr_br().set(r);
        });
        gpio_c7.bsrr_br().set_bit();
      }
      if cycle == pivot(counter, 0) {
        gpio_b14.bsrr_bs().set_bit();
      }
      if cycle == pivot(counter, STEP) {
        gpio_b7.bsrr_bs().set_bit();
      }
      if cycle == pivot(counter, STEP << 1) {
        gpio_c7.bsrr_bs().set_bit();
      }
      counter = if FAST.load(Relaxed) {
        counter.wrapping_add(0b100) & !0b011
      } else {
        counter.wrapping_add(0b001)
      };
    });
    Ok(())
  })
}

fn button_future(
  mut exti13: ExtiLn13<thr::Exti1510<Ltt>>,
  sys_tick: thr::SysTick<Ltt>,
) -> impl Future<Item = (), Error = !> {
  const DEBOUNCE_INTERVAL: u32 = 2000;

  fib::add(sys_tick, || loop {
    let debounce = DEBOUNCE.load(Relaxed);
    if debounce != 0 {
      DEBOUNCE.store(debounce - 1, Relaxed);
    }
    yield;
  });

  let stream = exti13.add_stream_skip();
  async(move || {
    await_for!(() in stream => {
      if DEBOUNCE.load(Relaxed) == 0 {
        FAST.store(!FAST.load(Relaxed), Relaxed);
        DEBOUNCE.store(DEBOUNCE_INTERVAL, Relaxed);
      }
    });
    Ok(())
  })
}
