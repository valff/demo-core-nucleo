//! The entry point.

use consts::{PLLCLK_FACTOR, PLL_INPUT_FACTOR, PLL_OUTPUT_FACTOR, SYS_TICK_SEC};
use core::sync::atomic::{AtomicBool, AtomicU32};
use core::sync::atomic::Ordering::*;
use drone_core::origin::OriginToken;
use drone_core::peripheral::PeripheralDevice;
use drone_cortex_m::peripherals::exti::ExtiLn13;
use drone_cortex_m::peripherals::gpio::{GpioPin, Gpiob14, Gpiob7, Gpioc13,
                                        Gpioc7};
use drone_cortex_m::peripherals::timer::{SysTick, Timer};
use drone_cortex_m::reg::{self, RegIndex};
use drone_cortex_m::reg::prelude::*;
use drone_cortex_m::thread::prelude::*;
use thread::{self, ThreadIndex};

static FAST: AtomicBool = AtomicBool::new(false);
static DEBOUNCE: AtomicU32 = AtomicU32::new(0);

/// The entry point.
pub fn origin(origin: OriginToken) {
  let regs = RegIndex::new(origin);
  let thrd = ThreadIndex::new(peripheral_nvic!(regs));
  let sys_tick = peripheral_sys_tick!(regs, thrd);
  let gpiob14 = peripheral_gpiob_14!(regs);
  let gpiob7 = peripheral_gpiob_7!(regs);
  let gpioc13 = peripheral_gpioc_13!(regs);
  let gpioc7 = peripheral_gpioc_7!(regs);
  let exti13 = peripheral_exti_ln_13!(regs, thrd);

  let RegIndex {
    rcc_ahb2enr,
    rcc_apb2enr,
    ..
  } = regs;

  // Panic on Hard Fault.
  thrd.hard_fault.routine_fn(|| panic!("Hard Fault"));

  // Configure maximum processor clock frequency of 80MHz.
  let init = hclk_init(
    thrd.nmi.into(),
    thrd.rcc,
    regs.flash_acr.into(),
    &mut regs.pwr_cr1.into(),
    &mut regs.rcc_apb1enr1.into(),
    &mut regs.rcc_bdcr.into(),
    regs.rcc_cfgr.into(),
    regs.rcc_cicr.into(),
    &mut regs.rcc_cier.into(),
    regs.rcc_cifr.into(),
    &mut regs.rcc_cr.into(),
    &mut regs.rcc_pllcfgr.into(),
  );

  let init = AsyncFuture::new(move || {
    await!(init)?;
    thrd.led.enable_batch(|r| {
      thrd.led.enable(r);
      thrd.button.enable(r);
    });
    thrd.exti15_10.enable_irq();
    // Configure push-button and LED pins.
    peripheral_init(
      &gpiob14,
      &gpiob7,
      &gpioc13,
      &gpioc7,
      &exti13,
      &mut rcc_ahb2enr.into(),
      &mut rcc_apb2enr.into(),
    );
    // Setup push-button routine.
    thrd
      .button
      .exec(button_future(exti13, thrd.sys_tick.into()));
    // Setup LED blink routine.
    thrd.led.exec(led_future(sys_tick, gpiob14, gpiob7, gpioc7));
    Ok::<(), !>(())
  });

  init.wait().ok();

  // Do not return to the reset handler from interrupts.
  regs.scb_scr.modify(|r| r.set_sleeponexit());
}

#[cfg_attr(feature = "clippy", allow(too_many_arguments))]
fn hclk_init(
  nmi: thread::Nmi<Ltt>,
  rcc: thread::Rcc<Ctt>,
  mut flash_acr: reg::flash::Acr<Urt>,
  pwr_cr1: &mut reg::pwr::Cr1<Urt>,
  rcc_apb1enr1: &mut reg::rcc::Apb1Enr1<Urt>,
  rcc_bdcr: &mut reg::rcc::Bdcr<Urt>,
  mut rcc_cfgr: reg::rcc::Cfgr<Urt>,
  mut rcc_cicr: reg::rcc::Cicr<Frt>,
  rcc_cier: &mut reg::rcc::Cier<Urt>,
  mut rcc_cifr: reg::rcc::Cifr<Frt>,
  rcc_cr: &mut reg::rcc::Cr<Urt>,
  rcc_pllcfgr: &mut reg::rcc::Pllcfgr<Urt>,
) -> impl Future<Item = (), Error = !> {
  // Enable on-board LSE crystal.
  rcc_apb1enr1.modify(|r| r.set_pwren());
  pwr_cr1.modify(|r| r.set_dbp());
  rcc_bdcr.modify(|r| r.set_lseon().clear_lsebyp().write_rtcsel(0b01));
  while !rcc_bdcr.load().lserdy() {}
  lse_css_failure(nmi, rcc_cicr.fork(), rcc_cifr.fork(), || {
    panic!("LSE clock failure")
  });
  rcc_cier.modify(|r| r.set_lsecssie());
  rcc_bdcr.modify(|r| r.set_lsecsson());

  // Configure MSI to use hardware auto calibration with LSE.
  rcc_cr.modify(|r| r.set_msipllen().set_msirgsel().write_msirange(0b0111));

  // Configure PLL to use MSI on 80MHz.
  rcc_pllcfgr.modify(|r| {
    r.write_pllsrc(0b01)
      .set_pllren()
      .write_pllr((PLLCLK_FACTOR >> 1) - 1)
      .write_pllm(PLL_INPUT_FACTOR - 1)
      .write_plln(PLL_OUTPUT_FACTOR)
  });

  pll_on(rcc, rcc_cicr, rcc_cier, rcc_cifr, rcc_cr).and_then(move |()| {
    // Setup flash to use at maximum performance.
    flash_acr.modify(|r| r.set_prften().set_icen().set_dcen().write_latency(2));
    rcc_cfgr.modify(|r| r.write_sw(0b11));
    Ok(())
  })
}

#[cfg_attr(feature = "clippy", allow(too_many_arguments))]
fn peripheral_init(
  gpiob14: &Gpiob14<Srt>,
  gpiob7: &Gpiob7<Srt>,
  gpioc13: &Gpioc13<Srt>,
  gpioc7: &Gpioc7<Srt>,
  exti13: &ExtiLn13<thread::Exti1510<Ltt>>,
  rcc_ahb2enr: &mut reg::rcc::Ahb2Enr<Urt>,
  rcc_apb2enr: &mut reg::rcc::Apb2Enr<Urt>,
) {
  rcc_apb2enr.modify(|r| r.set_syscfgen());
  rcc_ahb2enr.modify(|r| r.set_gpioben().set_gpiocen());
  exti13.exticr_exti().write_bits(0b0010);
  exti13.imr_mr().set_bit();
  exti13.rtsr_rt().set_bit();
  gpiob7.moder().modify(|r| {
    gpiob7.moder().write(r, 0b01);
    gpiob14.moder().write(r, 0b01);
  });
  gpioc7.moder().modify(|r| {
    gpioc7.moder().write(r, 0b01);
    gpioc13.moder().write(r, 0b00);
  });
  gpiob7.otyper().modify(|r| {
    gpiob7.otyper().clear(r);
    gpiob14.otyper().clear(r);
  });
  gpioc7.otyper().clear_bit();
  gpiob7.ospeedr().modify(|r| {
    gpiob7.ospeedr().write(r, 0b11);
    gpiob14.ospeedr().write(r, 0b11);
  });
  gpioc7.ospeedr().write_bits(0b11);
  gpioc13.pupdr().write_bits(0b10);
}

fn led_future(
  mut sys_tick: SysTick<thread::SysTick<Ltt>>,
  gpiob14: Gpiob14<Srt>,
  gpiob7: Gpiob7<Srt>,
  gpioc7: Gpioc7<Srt>,
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
  AsyncFuture::new(move || {
    await_for!(() in stream; {
      if counter & ((1 << 14) - 1) == 0 {
        println!("Counter: {}", counter);
      }
      let cycle = counter & ((1 << WIDTH) - 1);
      if cycle == 0 {
        gpiob7.bsrr_br().reset(|r| {
          gpiob7.bsrr_br().set(r);
          gpiob14.bsrr_br().set(r);
        });
        gpioc7.bsrr_br().set_bit();
      }
      if cycle == pivot(counter, 0) {
        gpiob14.bsrr_bs().set_bit();
      }
      if cycle == pivot(counter, STEP) {
        gpiob7.bsrr_bs().set_bit();
      }
      if cycle == pivot(counter, STEP << 1) {
        gpioc7.bsrr_bs().set_bit();
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
  mut exti13: ExtiLn13<thread::Exti1510<Ltt>>,
  sys_tick: thread::SysTick<Ltt>,
) -> impl Future<Item = (), Error = !> {
  const DEBOUNCE_INTERVAL: u32 = 2000;

  sys_tick.routine(|| loop {
    let debounce = DEBOUNCE.load(Relaxed);
    if debounce != 0 {
      DEBOUNCE.store(debounce - 1, Relaxed);
    }
    yield;
  });

  let stream = exti13.stream_skip();
  AsyncFuture::new(move || {
    await_for!(() in stream; {
      if DEBOUNCE.load(Relaxed) == 0 {
        FAST.store(!FAST.load(Relaxed), Relaxed);
        DEBOUNCE.store(DEBOUNCE_INTERVAL, Relaxed);
      }
    });
    Ok(())
  })
}

fn lse_css_failure<F>(
  nmi: thread::Nmi<Ltt>,
  rcc_cicr: reg::rcc::Cicr<Frt>,
  rcc_cifr: reg::rcc::Cifr<Frt>,
  f: F,
) where
  F: FnOnce() + Send + 'static,
{
  nmi.routine(move || loop {
    if rcc_cifr.load().lsecssf() {
      rcc_cicr.reset(|r| r.set_lsecssc());
      break f();
    }
    yield;
  });
}

fn pll_on(
  rcc: thread::Rcc<Ctt>,
  rcc_cicr: reg::rcc::Cicr<Frt>,
  rcc_cier: &mut reg::rcc::Cier<Urt>,
  rcc_cifr: reg::rcc::Cifr<Frt>,
  rcc_cr: &mut reg::rcc::Cr<Urt>,
) -> impl Future<Item = (), Error = !> {
  rcc.enable_irq();
  rcc_cier.modify(|r| r.set_pllrdyie());
  let ready = rcc.future(move || loop {
    if rcc_cifr.load().pllrdyf() {
      rcc_cicr.reset(|r| r.set_pllrdyc());
      break Ok(());
    }
    yield;
  });
  rcc_cr.modify(|r| r.set_pllon());
  ready
}
