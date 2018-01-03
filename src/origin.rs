//! The entry point.

use consts::{PLLCLK_FACTOR, PLL_INPUT_FACTOR, PLL_OUTPUT_FACTOR, SYS_TICK_SEC};
use core::sync::atomic::{AtomicBool, AtomicU32};
use core::sync::atomic::Ordering::*;
use drone_cortex_m::peripherals::exti::{ExtiLn, ExtiLn13, ExtiLnConf,
                                        ExtiLnExt};
use drone_cortex_m::peripherals::gpio::{GpioPin, Gpiob14, Gpiob7, Gpioc13,
                                        Gpioc7};
use drone_cortex_m::peripherals::timer::Timer;
use drone_cortex_m::reg::{flash, nvic, pwr, rcc, RegIndex};
use drone_cortex_m::reg::prelude::*;
use drone_cortex_m::thread::prelude::*;
use thread::{self, ThreadIndex, ThreadLocal};

static FAST: AtomicBool = AtomicBool::new(false);
static DEBOUNCE: AtomicU32 = AtomicU32::new(0);

/// The entry point.
pub fn origin(thrd: ThreadIndex, regs: RegIndex) {
  let mut sys_tick = peripheral_sys_tick!(thrd, regs);
  let gpiob14 = peripheral_gpiob_14!(regs);
  let gpiob7 = peripheral_gpiob_7!(regs);
  let gpioc13 = peripheral_gpioc_13!(regs);
  let gpioc7 = peripheral_gpioc_7!(regs);
  let exti13 = peripheral_exti_ln_13!(thrd, regs);

  let RegIndex {
    nvic_iser1,
    rcc_ahb2enr,
    rcc_apb2enr,
    ..
  } = regs;

  // Panic on Hard Fault.
  thrd.hard_fault.routine_fn(|| panic!("Hard Fault"));

  // Configure maximum processor clock frequency of 80MHz.
  let init = hclk_init(
    thrd.nmi,
    thrd.rcc,
    regs.flash_acr.into(),
    &mut regs.nvic_iser0.into(),
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
    // Configure push-button and LED pins.
    peripheral_init(
      &gpiob14,
      &gpiob7,
      &gpioc13,
      &gpioc7,
      &exti13,
      &mut nvic_iser1.into(),
      &mut rcc_ahb2enr.into(),
      &mut rcc_apb2enr.into(),
    );
    // Setup LED blink routine.
    led_routine(thrd.sys_tick, gpiob14, gpiob7, gpioc7);
    // Setup push-button routine.
    button_routine(exti13, thrd.sys_tick);
    // Setup SysTick timer.
    let ctrl = sys_tick.ctrl().default().val();
    let stream = sys_tick.interval(SYS_TICK_SEC / 15_000, ctrl);
    Ok::<_, ()>(stream)
  });

  if let Ok(stream) = init.wait() {
    for _ in stream.wait() {}
  }

  // Do not return to the reset handler from interrupts.
  regs.scb_scr.modify(|r| r.set_sleeponexit());
}

#[cfg_attr(feature = "clippy", allow(too_many_arguments))]
fn hclk_init(
  nmi: ThreadToken<ThreadLocal, thread::Nmi>,
  rcc: ThreadToken<ThreadLocal, thread::Rcc>,
  mut flash_acr: flash::Acr<Utt>,
  nvic_iser0: &mut nvic::Iser0<Utt>,
  pwr_cr1: &mut pwr::Cr1<Utt>,
  rcc_apb1enr1: &mut rcc::Apb1Enr1<Utt>,
  rcc_bdcr: &mut rcc::Bdcr<Utt>,
  mut rcc_cfgr: rcc::Cfgr<Utt>,
  mut rcc_cicr: rcc::Cicr<Ftt>,
  rcc_cier: &mut rcc::Cier<Utt>,
  mut rcc_cifr: rcc::Cifr<Ftt>,
  rcc_cr: &mut rcc::Cr<Utt>,
  rcc_pllcfgr: &mut rcc::Pllcfgr<Utt>,
) -> impl Future<Item = (), Error = ()> {
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

  let pll_on = pll_on(rcc, nvic_iser0, rcc_cicr, rcc_cier, rcc_cifr, rcc_cr);
  AsyncFuture::new(move || {
    await!(pll_on)?;
    // Setup flash to use at maximum performance.
    flash_acr.modify(|r| r.set_prften().set_icen().set_dcen().write_latency(2));
    rcc_cfgr.modify(|r| r.write_sw(0b11));
    Ok(())
  })
}

#[cfg_attr(feature = "clippy", allow(too_many_arguments))]
fn peripheral_init(
  gpiob14: &Gpiob14<Stt>,
  gpiob7: &Gpiob7<Stt>,
  gpioc13: &Gpioc13<Stt>,
  gpioc7: &Gpioc7<Stt>,
  exti13: &ExtiLn13<ThreadLocal, thread::Exti1510>,
  nvic_iser1: &mut nvic::Iser1<Utt>,
  rcc_ahb2enr: &mut rcc::Ahb2Enr<Utt>,
  rcc_apb2enr: &mut rcc::Apb2Enr<Utt>,
) {
  nvic_iser1.modify(|r| {
    let setena = r.setena();
    r.write_setena(setena | 1 << thread::Exti1510::INTERRUPT_NUMBER - 32)
  });
  rcc_apb2enr.modify(|r| r.set_syscfgen());
  rcc_ahb2enr.modify(|r| r.set_gpioben().set_gpiocen());
  exti13.exticr().write_bits(0b0010);
  exti13.imr().set_bit();
  exti13.rtsr().set_bit();
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

fn led_routine(
  sys_tick: ThreadToken<ThreadLocal, thread::SysTick>,
  gpiob14: Gpiob14<Stt>,
  gpiob7: Gpiob7<Stt>,
  gpioc7: Gpioc7<Stt>,
) {
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

  sys_tick.routine(move || loop {
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
    yield;
  });
}

fn button_routine(
  exti13: ExtiLn13<ThreadLocal, thread::Exti1510>,
  sys_tick: ThreadToken<ThreadLocal, thread::SysTick>,
) {
  const DEBOUNCE_INTERVAL: u32 = 2000;

  on_exti13(exti13, || {
    if DEBOUNCE.load(Relaxed) == 0 {
      FAST.store(!FAST.load(Relaxed), Relaxed);
      DEBOUNCE.store(DEBOUNCE_INTERVAL, Relaxed);
    }
  });

  sys_tick.routine(|| loop {
    let debounce = DEBOUNCE.load(Relaxed);
    if debounce != 0 {
      DEBOUNCE.store(debounce - 1, Relaxed);
    }
    yield;
  });
}

fn on_exti13<F>(exti13: ExtiLn13<ThreadLocal, thread::Exti1510>, f: F)
where
  F: Fn() + Send + 'static,
{
  exti13.irq().routine(move || loop {
    if exti13.pr().read_bit_band() {
      exti13.pr().set_bit_band();
      f();
    }
    yield;
  });
}

fn lse_css_failure<F>(
  nmi: ThreadToken<ThreadLocal, thread::Nmi>,
  rcc_cicr: rcc::Cicr<Ftt>,
  rcc_cifr: rcc::Cifr<Ftt>,
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
  rcc: ThreadToken<ThreadLocal, thread::Rcc>,
  nvic_iser0: &mut nvic::Iser0<Utt>,
  rcc_cicr: rcc::Cicr<Ftt>,
  rcc_cier: &mut rcc::Cier<Utt>,
  rcc_cifr: rcc::Cifr<Ftt>,
  rcc_cr: &mut rcc::Cr<Utt>,
) -> impl Future<Item = (), Error = ()> {
  nvic_iser0.modify(|r| {
    let setena = r.setena();
    r.write_setena(setena | 1 << thread::Rcc::INTERRUPT_NUMBER)
  });
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
