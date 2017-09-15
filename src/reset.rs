//! Reset routine.

use consts::{PLLCLK_FACTOR, PLL_INPUT_FACTOR, PLL_OUTPUT_FACTOR, SYS_TICK_SEC};
use drone::mem;
use drone_stm32::{itm, reg};
use drone_stm32::reg::prelude::*;
use vtable;

const WIDTH: u32 = 5;
const SPEED: u32 = 1;

/// The program entry point.
pub fn main() {
  // Begin of register bindings definition.
  let flash_acr = unsafe { reg::flash::Acr::<Lr>::bind() };
  let gpiob_bsrr = unsafe { reg::gpiob::Bsrr::<Lr>::bind() };
  let gpiob_moder = unsafe { reg::gpiob::Moder::<Lr>::bind() };
  let gpiob_ospeedr = unsafe { reg::gpiob::Ospeedr::<Lr>::bind() };
  let gpiob_otyper = unsafe { reg::gpiob::Otyper::<Lr>::bind() };
  let gpioc_bsrr = unsafe { reg::gpioc::Bsrr::<Lr>::bind() };
  let gpioc_moder = unsafe { reg::gpioc::Moder::<Lr>::bind() };
  let gpioc_ospeedr = unsafe { reg::gpioc::Ospeedr::<Lr>::bind() };
  let gpioc_otyper = unsafe { reg::gpioc::Otyper::<Lr>::bind() };
  let nvic_iser0 = unsafe { reg::nvic::Iser0::<Lr>::bind() };
  let pwr_cr1 = unsafe { reg::pwr::Cr1::<Lr>::bind() };
  let rcc_ahb2enr = unsafe { reg::rcc::Ahb2Enr::<Lr>::bind() };
  let rcc_apb1enr1 = unsafe { reg::rcc::Apb1Enr1::<Lr>::bind() };
  let rcc_bdcr = unsafe { reg::rcc::Bdcr::<Lr>::bind() };
  let rcc_cfgr = unsafe { reg::rcc::Cfgr::<Lr>::bind() };
  let rcc_cicr = unsafe { reg::rcc::Cicr::<Ar>::bind() };
  let rcc_cier = unsafe { reg::rcc::Cier::<Lr>::bind() };
  let rcc_cifr = unsafe { reg::rcc::Cifr::<Ar>::bind() };
  let rcc_cr = unsafe { reg::rcc::Cr::<Lr>::bind() };
  let rcc_pllcfgr = unsafe { reg::rcc::Pllcfgr::<Lr>::bind() };
  let scb_scr = unsafe { reg::scb::Scr::<Lr>::bind() };
  let stk_ctrl = unsafe { reg::stk::Ctrl::<Lr>::bind() };
  let stk_load = unsafe { reg::stk::Load::<Lr>::bind() };
  // End of register bindings definition.

  // Important initialization before all other code.
  unsafe {
    mem::bss_init();
    mem::data_init();
    mem::heap_init();
    itm::init();
  }

  // Panic on Hard Fault.
  vtable::hard_fault().push_callback(move || {
    panic!("Hard Fault");
  });

  // Panic on LSE failure.
  vtable::nmi().push(move || loop {
    if rcc_cifr.read().lsecssf() {
      rcc_cicr.write(|reg| reg.set_lsecssc(true));
      panic!("LSE clock failure");
    }
    yield;
  });

  // Enable on-board LSE crystal.
  rcc_apb1enr1.modify(|reg| reg.set_pwren(true));
  pwr_cr1.modify(|reg| reg.set_dbp(true));
  rcc_bdcr.modify(|reg| reg.set_lseon(true).set_lsebyp(false).set_rtcsel(0b01));
  while !rcc_bdcr.read().lserdy() {}
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
  nvic_iser0.modify(|reg| reg.set_bit(5, true));
  rcc_cier.modify(|reg| reg.set_pllrdyie(true));
  vtable::rcc().push(move || loop {
    // When PLL is ready.
    if rcc_cifr.read().pllrdyf() {
      rcc_cicr.write(|reg| reg.set_pllrdyc(true));
      // Setup flash to use at maximum performance.
      flash_acr.modify(|reg| {
        reg
          .set_prften(true)
          .set_icen(true)
          .set_dcen(true)
          .set_latency(2)
      });
      rcc_cfgr.modify(|reg| reg.set_sw(0b11));

      // Configure LED pins.
      rcc_ahb2enr.modify(|reg| reg.set_gpioben(true).set_gpiocen(true));
      gpiob_moder.modify(|reg| reg.set_moder7(0b01).set_moder14(0b01));
      gpioc_moder.modify(|reg| reg.set_moder7(0b01));
      gpiob_otyper.modify(|reg| reg.set_ot7(false).set_ot14(false));
      gpioc_otyper.modify(|reg| reg.set_ot7(false));
      gpiob_ospeedr.modify(|reg| reg.set_ospeedr7(0b11).set_ospeedr14(0b11));
      gpioc_ospeedr.modify(|reg| reg.set_ospeedr7(0b11));

      // Setup counter routine.
      stk_load.write(|reg| reg.set_reload(SYS_TICK_SEC / 2048));
      let mut counter = ((0b1 << (WIDTH * 2)) << SPEED) - 1;
      vtable::sys_tick().push(move || loop {
        let lightness = counter >> WIDTH >> SPEED;
        let position = counter & ((0b1 << WIDTH) - 1);
        if lightness == position {
          gpiob_bsrr.write(|reg| reg.set_br7(true).set_bs14(true));
          gpioc_bsrr.write(|reg| reg.set_bs7(true));
        } else if position == 0 {
          gpiob_bsrr.write(|reg| reg.set_bs7(true).set_br14(true));
          gpioc_bsrr.write(|reg| reg.set_br7(true));
        }
        if counter == 0 {
          panic!(); // reboots
        } else {
          counter -= 1;
        }
        yield;
      });
      stk_ctrl.modify(|reg| reg.set_enable(true).set_tickint(true));
      break;
    }
    yield;
  });
  rcc_cr.modify(|reg| reg.set_pllon(true));

  // Not to return to the reset handler from interrupts.
  scb_scr.modify(|reg| reg.set_sleeponexit(true));
}
