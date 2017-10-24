//! The reset handler.

use consts::{PLLCLK_FACTOR, PLL_INPUT_FACTOR, PLL_OUTPUT_FACTOR, SYS_TICK_SEC};
use core::sync::atomic::{AtomicBool, AtomicU32};
use core::sync::atomic::Ordering::*;
use drone::mem;
use drone::thread::Thread;
use drone_cortex_m::{itm, mcu, reg};
use drone_cortex_m::reg::prelude::*;
use futures::Future;
use heap;
use thread;

extern "C" {
  static mut BSS_START: usize;
  static BSS_END: usize;
  static mut DATA_START: usize;
  static DATA_END: usize;
  static DATA_CONST: usize;
  static mut HEAP_START: usize;
}

static FAST: AtomicBool = AtomicBool::new(false);
static DEBOUNCE: AtomicU32 = AtomicU32::new(0);

/// The program entry point.
pub fn main() {
  // Define register bindings.
  reg::bind! {
    exti_imr1: reg::exti::Imr1<Lr>,
    exti_pr1: reg::exti::Pr1<Lr>,
    exti_rtsr1: reg::exti::Rtsr1<Lr>,
    flash_acr: reg::flash::Acr<Lr>,
    gpiob_bsrr: reg::gpiob::Bsrr<Lr>,
    gpiob_moder: reg::gpiob::Moder<Lr>,
    gpiob_ospeedr: reg::gpiob::Ospeedr<Lr>,
    gpiob_otyper: reg::gpiob::Otyper<Lr>,
    gpioc_bsrr: reg::gpioc::Bsrr<Lr>,
    gpioc_moder: reg::gpioc::Moder<Lr>,
    gpioc_ospeedr: reg::gpioc::Ospeedr<Lr>,
    gpioc_otyper: reg::gpioc::Otyper<Lr>,
    gpioc_pupdr: reg::gpioc::Pupdr<Lr>,
    nvic_iser0: reg::nvic::Iser0<Lr>,
    nvic_iser1: reg::nvic::Iser1<Lr>,
    pwr_cr1: reg::pwr::Cr1<Lr>,
    rcc_ahb2enr: reg::rcc::Ahb2Enr<Lr>,
    rcc_apb1enr1: reg::rcc::Apb1Enr1<Lr>,
    rcc_apb2enr: reg::rcc::Apb2Enr<Lr>,
    rcc_bdcr: reg::rcc::Bdcr<Lr>,
    rcc_cfgr: reg::rcc::Cfgr<Lr>,
    rcc_cicr: reg::rcc::Cicr<Ar>,
    rcc_cier: reg::rcc::Cier<Lr>,
    rcc_cifr: reg::rcc::Cifr<Ar>,
    rcc_cr: reg::rcc::Cr<Lr>,
    rcc_pllcfgr: reg::rcc::Pllcfgr<Lr>,
    scb_scr: reg::scb::Scr<Lr>,
    stk_ctrl: reg::stk::Ctrl<Lr>,
    stk_load: reg::stk::Load<Lr>,
    syscfg_exticr4: reg::syscfg::Exticr4<Lr>,
  }

  // Subsystems initialization.
  unsafe {
    itm::init();
    mem::bss_init(&mut BSS_START, &BSS_END);
    mem::data_init(&mut DATA_START, &DATA_END, &DATA_CONST);
    heap::init(&mut HEAP_START);
    thread::init();
  }

  // Panic on Hard Fault.
  thread::hard_fault().callback(|| panic!("Hard Fault"));

  // Configure maximum processor clock frequency of 80MHz.
  let hclk_ready = hclk_init(
    flash_acr,
    &nvic_iser0,
    &pwr_cr1,
    &rcc_apb1enr1,
    &rcc_bdcr,
    rcc_cfgr,
    rcc_cicr,
    &rcc_cier,
    rcc_cifr,
    &rcc_cr,
    &rcc_pllcfgr,
  );

  mcu::wait_for(hclk_ready.and_then(move |_| {
    // Configure push-button and LED pins.
    gpio_init(
      &exti_imr1,
      &exti_rtsr1,
      &gpiob_moder,
      &gpiob_ospeedr,
      &gpiob_otyper,
      &gpioc_moder,
      &gpioc_ospeedr,
      &gpioc_otyper,
      &gpioc_pupdr,
      &nvic_iser1,
      &rcc_ahb2enr,
      &rcc_apb2enr,
      &syscfg_exticr4,
    );
    // Setup LED blink routine.
    led_routine(gpiob_bsrr, gpioc_bsrr);
    // Setup push-button routine.
    button_routine(exti_pr1);
    // Setup SysTick timer.
    stk_load.write(|reg| reg.set_reload(SYS_TICK_SEC / 15_000));
    stk_ctrl.modify(|reg| reg.set_enable(true).set_tickint(true));
    Ok(())
  })).ok();

  // Do not return to the reset handler from interrupts.
  scb_scr.modify(|reg| reg.set_sleeponexit(true));
}

#[cfg_attr(feature = "clippy", allow(too_many_arguments))]
fn hclk_init(
  flash_acr: reg::flash::Acr<Lr>,
  nvic_iser0: &reg::nvic::Iser0<Lr>,
  pwr_cr1: &reg::pwr::Cr1<Lr>,
  rcc_apb1enr1: &reg::rcc::Apb1Enr1<Lr>,
  rcc_bdcr: &reg::rcc::Bdcr<Lr>,
  rcc_cfgr: reg::rcc::Cfgr<Lr>,
  rcc_cicr: reg::rcc::Cicr<Ar>,
  rcc_cier: &reg::rcc::Cier<Lr>,
  rcc_cifr: reg::rcc::Cifr<Ar>,
  rcc_cr: &reg::rcc::Cr<Lr>,
  rcc_pllcfgr: &reg::rcc::Pllcfgr<Lr>,
) -> impl Future<Item = (), Error = ()> {
  // Enable on-board LSE crystal.
  rcc_apb1enr1.modify(|reg| reg.set_pwren(true));
  pwr_cr1.modify(|reg| reg.set_dbp(true));
  rcc_bdcr.modify(|reg| reg.set_lseon(true).set_lsebyp(false).set_rtcsel(0b01));
  while !rcc_bdcr.read().lserdy() {}
  lse_css_failure(rcc_cicr, rcc_cifr, || panic!("LSE clock failure"));
  rcc_cier.modify(|reg| reg.set_lsecssie(true));
  rcc_bdcr.modify(|reg| reg.set_lsecsson(true));

  // Configure MSI to use hardware auto calibration with LSE.
  rcc_cr.modify(|reg| {
    reg
      .set_msipllen(true)
      .set_msirgsel(true)
      .set_msirange(0b0111)
  });

  // Configure PLL to use MSI on 80MHz.
  rcc_pllcfgr.modify(|reg| {
    reg
      .set_pllsrc(0b01)
      .set_pllren(true)
      .set_pllr((PLLCLK_FACTOR >> 1) - 1)
      .set_pllm(PLL_INPUT_FACTOR - 1)
      .set_plln(PLL_OUTPUT_FACTOR)
  });

  pll_on(nvic_iser0, rcc_cicr, rcc_cier, rcc_cifr, rcc_cr).and_then(move |_| {
    // Setup flash to use at maximum performance.
    flash_acr.modify(|reg| {
      reg
        .set_prften(true)
        .set_icen(true)
        .set_dcen(true)
        .set_latency(2)
    });
    rcc_cfgr.modify(|reg| reg.set_sw(0b11));
    Ok(())
  })
}

#[cfg_attr(feature = "clippy", allow(too_many_arguments))]
fn gpio_init(
  exti_imr1: &reg::exti::Imr1<Lr>,
  exti_rtsr1: &reg::exti::Rtsr1<Lr>,
  gpiob_moder: &reg::gpiob::Moder<Lr>,
  gpiob_ospeedr: &reg::gpiob::Ospeedr<Lr>,
  gpiob_otyper: &reg::gpiob::Otyper<Lr>,
  gpioc_moder: &reg::gpioc::Moder<Lr>,
  gpioc_ospeedr: &reg::gpioc::Ospeedr<Lr>,
  gpioc_otyper: &reg::gpioc::Otyper<Lr>,
  gpioc_pupdr: &reg::gpioc::Pupdr<Lr>,
  nvic_iser1: &reg::nvic::Iser1<Lr>,
  rcc_ahb2enr: &reg::rcc::Ahb2Enr<Lr>,
  rcc_apb2enr: &reg::rcc::Apb2Enr<Lr>,
  syscfg_exticr4: &reg::syscfg::Exticr4<Lr>,
) {
  nvic_iser1.modify(|reg| reg.set_bit(8, true));
  rcc_apb2enr.modify(|reg| reg.set_syscfgen(true));
  rcc_ahb2enr.modify(|reg| reg.set_gpioben(true).set_gpiocen(true));
  syscfg_exticr4.modify(|reg| reg.set_exti13(0b0010));
  exti_imr1.modify(|reg| reg.set_mr13(true));
  exti_rtsr1.modify(|reg| reg.set_tr13(true));
  gpiob_moder.modify(|reg| reg.set_moder7(0b01).set_moder14(0b01));
  gpioc_moder.modify(|reg| reg.set_moder7(0b01).set_moder13(0b00));
  gpiob_otyper.modify(|reg| reg.set_ot7(false).set_ot14(false));
  gpioc_otyper.modify(|reg| reg.set_ot7(false));
  gpiob_ospeedr.modify(|reg| reg.set_ospeedr7(0b11).set_ospeedr14(0b11));
  gpioc_ospeedr.modify(|reg| reg.set_ospeedr7(0b11));
  gpioc_pupdr.modify(|reg| reg.set_pupdr13(0b10));
}

fn led_routine(
  gpiob_bsrr: reg::gpiob::Bsrr<Lr>,
  gpioc_bsrr: reg::gpioc::Bsrr<Lr>,
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

  thread::sys_tick().routine(move || loop {
    if counter & ((1 << 14) - 1) == 0 {
      iprintln!("Counter: {}", counter);
    }
    let cycle = counter & ((1 << WIDTH) - 1);
    if cycle == 0 {
      gpiob_bsrr.write(|reg| reg.set_br7(true).set_br14(true));
      gpioc_bsrr.write(|reg| reg.set_br7(true));
    }
    if cycle == pivot(counter, 0) {
      gpiob_bsrr.write(|reg| reg.set_bs14(true));
    }
    if cycle == pivot(counter, STEP) {
      gpiob_bsrr.write(|reg| reg.set_bs7(true));
    }
    if cycle == pivot(counter, STEP << 1) {
      gpioc_bsrr.write(|reg| reg.set_bs7(true));
    }
    counter = if FAST.load(Relaxed) {
      counter.wrapping_add(0b100) & !0b011
    } else {
      counter.wrapping_add(0b001)
    };
    yield;
  });
}

fn button_routine(exti_pr1: reg::exti::Pr1<Lr>) {
  const DEBOUNCE_INTERVAL: u32 = 2000;

  on_exti13(exti_pr1, || if DEBOUNCE.load(Relaxed) == 0 {
    FAST.store(!FAST.load(Relaxed), Relaxed);
    DEBOUNCE.store(DEBOUNCE_INTERVAL, Relaxed);
  });

  thread::sys_tick().routine(|| loop {
    let debounce = DEBOUNCE.load(Relaxed);
    if debounce != 0 {
      DEBOUNCE.store(debounce - 1, Relaxed);
    }
    yield;
  });
}

fn on_exti13<F>(exti_pr1: reg::exti::Pr1<Lr>, f: F)
where
  F: Fn() + Send + 'static,
{
  thread::exti15_10().routine(move || loop {
    if exti_pr1.read().pr13() {
      exti_pr1.write(|reg| reg.set_pr13(true));
      f();
    }
    yield;
  });
}

fn lse_css_failure<F>(
  rcc_cicr: reg::rcc::Cicr<Ar>,
  rcc_cifr: reg::rcc::Cifr<Ar>,
  f: F,
) where
  F: FnOnce() + Send + 'static,
{
  thread::nmi().routine(move || loop {
    if rcc_cifr.read().lsecssf() {
      rcc_cicr.write(|reg| reg.set_lsecssc(true));
      break f();
    }
    yield;
  });
}

fn pll_on(
  nvic_iser0: &reg::nvic::Iser0<Lr>,
  rcc_cicr: reg::rcc::Cicr<Ar>,
  rcc_cier: &reg::rcc::Cier<Lr>,
  rcc_cifr: reg::rcc::Cifr<Ar>,
  rcc_cr: &reg::rcc::Cr<Lr>,
) -> impl Future<Item = (), Error = ()> {
  nvic_iser0.modify(|reg| reg.set_bit(5, true));
  rcc_cier.modify(|reg| reg.set_pllrdyie(true));
  let ready = thread::rcc().future(move || loop {
    if rcc_cifr.read().pllrdyf() {
      rcc_cicr.write(|reg| reg.set_pllrdyc(true));
      break Ok(());
    }
    yield;
  });
  rcc_cr.modify(|reg| reg.set_pllon(true));
  ready
}
