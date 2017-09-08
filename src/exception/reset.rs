//! Invoked on power up and warm reset.

use VectorTable;
use consts::{PLLCLK_FACTOR, PLL_INPUT_FACTOR, PLL_OUTPUT_FACTOR, SYS_TICK_SEC};
use drone::exception::ExceptionTable;
use drone::memory;
use drone_stm32::{itm, util};
use drone_stm32::reg::{FlashAcr, GpiobBsrr, GpiobModer, GpiobOspeedr,
                       GpiobOtyper, GpiocBsrr, GpiocModer, GpiocOspeedr,
                       GpiocOtyper, PwrCr1, RccAhb2Enr, RccApb1Enr1, RccBdcr,
                       RccCfgr, RccCicr, RccCier, RccCifr, RccCr, RccPllcfgr,
                       StkCtrl, StkLoad};
use drone_stm32::reg::prelude::*;
use exception::{HardFaultConfig, NmiConfig, SysTickConfig};

/// The exception handler.
#[naked]
pub extern "C" fn handler() -> ! {
  // NOTE For each register delegate in this scope should exists exactly one
  // instance.
  let stk_ctrl = unsafe { StkCtrl::attach() };
  let stk_load = unsafe { StkLoad::attach() };
  let pwr_cr1 = unsafe { PwrCr1::attach() };
  let flash_acr = unsafe { FlashAcr::attach() };
  let rcc_ahb2enr = unsafe { RccAhb2Enr::attach() };
  let rcc_apb1enr1 = unsafe { RccApb1Enr1::attach() };
  let rcc_cr = unsafe { RccCr::attach() };
  let rcc_cfgr = unsafe { RccCfgr::attach() };
  let rcc_cier = unsafe { RccCier::attach() };
  let rcc_cifr = unsafe { RccCifr::attach() };
  let rcc_cicr = unsafe { RccCicr::attach() };
  let rcc_pllcfgr = unsafe { RccPllcfgr::attach() };
  let rcc_bdcr = unsafe { RccBdcr::attach() };
  let gpiob_moder = unsafe { GpiobModer::attach() };
  let gpioc_moder = unsafe { GpiocModer::attach() };
  let gpiob_otyper = unsafe { GpiobOtyper::attach() };
  let gpioc_otyper = unsafe { GpiocOtyper::attach() };
  let gpiob_ospeedr = unsafe { GpiobOspeedr::attach() };
  let gpioc_ospeedr = unsafe { GpiocOspeedr::attach() };
  let gpiob_bsrr = unsafe { GpiobBsrr::attach() };
  let gpioc_bsrr = unsafe { GpiocBsrr::attach() };

  unsafe {
    memory::bss_init();
    memory::data_init();
    itm::init();
    clock_init(
      pwr_cr1,
      flash_acr,
      rcc_apb1enr1,
      rcc_cr,
      rcc_cfgr,
      rcc_cier,
      rcc_pllcfgr,
      rcc_bdcr,
    );
    peripheral_init(
      rcc_ahb2enr,
      gpiob_moder,
      gpioc_moder,
      gpiob_otyper,
      gpioc_otyper,
      gpiob_ospeedr,
      gpioc_ospeedr,
    );
    VectorTable::config(move || {
      (
        NmiConfig { rcc_cifr, rcc_cicr },
        HardFaultConfig {},
        SysTickConfig {
          gpiob_bsrr,
          gpioc_bsrr,
        },
      )
    });
    exceptions_init(stk_ctrl, stk_load);
  }

  loop {
    util::wait_for_interrupt();
  }
}

#[cfg_attr(feature = "clippy", allow(too_many_arguments))]
unsafe fn clock_init(
  mut pwr_cr1: PwrCr1<Local>,
  mut flash_acr: FlashAcr<Local>,
  mut rcc_apb1enr1: RccApb1Enr1<Local>,
  mut rcc_cr: RccCr<Local>,
  mut rcc_cfgr: RccCfgr<Local>,
  mut rcc_cier: RccCier<Local>,
  mut rcc_pllcfgr: RccPllcfgr<Local>,
  mut rcc_bdcr: RccBdcr<Local>,
) {
  rcc_apb1enr1.modify(|reg| reg.set_pwren(true));
  pwr_cr1.modify(|reg| reg.set_dbp(true));
  rcc_bdcr.modify(|reg| reg.set_lseon(true).set_lsebyp(false).set_rtcsel(0b01));
  while !rcc_bdcr.read().lserdy() {}
  rcc_bdcr.modify(|reg| reg.set_lsecsson(true));
  rcc_cier.modify(|reg| reg.set_lsecssie(true));
  rcc_cr.modify(|reg| {
    reg
      .set_msipllen(true)
      .set_msirgsel(true)
      .set_msirange(0b0111)
  });
  rcc_pllcfgr.modify(|reg| {
    reg
      .set_pllsrc(0b01)
      .set_pllren(true)
      .set_pllr((PLLCLK_FACTOR >> 1) - 1)
      .set_pllm(PLL_INPUT_FACTOR - 1)
      .set_plln(PLL_OUTPUT_FACTOR)
  });
  rcc_cr.modify(|reg| reg.set_pllon(true));
  while !rcc_cr.read().pllrdy() {}
  flash_acr.modify(|reg| {
    reg
      .set_prften(true)
      .set_icen(true)
      .set_dcen(true)
      .set_latency(2)
  });
  rcc_cfgr.modify(|reg| reg.set_sw(0b11));
}

unsafe fn peripheral_init(
  mut rcc_ahb2enr: RccAhb2Enr<Local>,
  mut gpiob_moder: GpiobModer<Local>,
  mut gpioc_moder: GpiocModer<Local>,
  mut gpiob_otyper: GpiobOtyper<Local>,
  mut gpioc_otyper: GpiocOtyper<Local>,
  mut gpiob_ospeedr: GpiobOspeedr<Local>,
  mut gpioc_ospeedr: GpiocOspeedr<Local>,
) {
  rcc_ahb2enr.modify(|reg| reg.set_gpioben(true).set_gpiocen(true));
  gpiob_moder.modify(|reg| reg.set_moder7(0b01).set_moder14(0b01));
  gpioc_moder.modify(|reg| reg.set_moder7(0b01));
  gpiob_otyper.modify(|reg| reg.set_ot7(false).set_ot14(false));
  gpioc_otyper.modify(|reg| reg.set_ot7(false));
  gpiob_ospeedr.modify(|reg| reg.set_ospeedr7(0b11).set_ospeedr14(0b11));
  gpioc_ospeedr.modify(|reg| reg.set_ospeedr7(0b11));
}

unsafe fn exceptions_init(
  mut stk_ctrl: StkCtrl<Local>,
  mut stk_load: StkLoad<Local>,
) {
  stk_load.write_with(|reg| reg.set_reload(SYS_TICK_SEC / 2048));
  stk_ctrl.modify(|reg| reg.set_enable(true).set_tickint(true));
}
