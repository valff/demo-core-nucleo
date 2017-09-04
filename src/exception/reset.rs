//! Invoked on power up and warm reset.

use VectorTable;
use consts::{PLLCLK_FACTOR, PLL_INPUT_FACTOR, PLL_OUTPUT_FACTOR, SYS_TICK_SEC};
use drone::{itm, memory, util};
use drone::exception::ExceptionTable;
use drone::reg::{stk, AliasPointer, Delegate, Reg, Sreg, ValuePointer};
use drone::reg::flash::{self, AcrBits};
use drone::reg::gpio::{self, Mode, ModerPin, Ospeed, OspeedrPin, Otype,
                       OtyperPin};
use drone::reg::pwr::{self, Cr1Bits};
use drone::reg::rcc::{self, Ahb2enrBits, Ahb2enrIop, Apb1enr1Bits, BdcrBits,
                      BdcrRtcClock, CfgrSystemClock, CierBits, CrBits,
                      CrMsiRange, PllcfgrBits, PllcfgrPllSource};
use exception::{HardFaultConfig, NmiConfig, SysTickConfig};

/// The exception handler.
#[naked]
pub extern "C" fn handler() -> ! {
  // NOTE For each register delegate in this scope should exists exactly one
  // instance.
  let dbg_mcucr = Sreg::new();
  let dbg_demcr = Sreg::new();
  let dbg_tpiuspp = Sreg::new();
  let dbg_tpiuffc = Sreg::new();
  let dbg_itmla = Sreg::new();
  let dbg_itmtc = Sreg::new();
  let dbg_itmtp = Sreg::new();
  let stk_ctrl = Sreg::new();
  let stk_load = Sreg::new();
  let pwr_cr1 = Sreg::new();
  let flash_acr = Sreg::new();
  let rcc_ahb2enr = Sreg::new();
  let rcc_apb1enr1 = Sreg::new();
  let rcc_cr = Sreg::new();
  let rcc_cfgr = Sreg::new();
  let rcc_cier = Sreg::new();
  let rcc_cifr = Sreg::new();
  let rcc_cicr = Sreg::new();
  let rcc_pllcfgr = Sreg::new();
  let rcc_bdcr = Sreg::new();
  let gpiob_moder = Sreg::new();
  let gpioc_moder = Sreg::new();
  let gpiob_otyper = Sreg::new();
  let gpioc_otyper = Sreg::new();
  let gpiob_ospeedr = Sreg::new();
  let gpioc_ospeedr = Sreg::new();
  let gpiob_cbsrr = Sreg::new();
  let gpioc_cbsrr = Sreg::new();

  unsafe {
    memory::bss_init();
    memory::data_init();
    itm::init(
      &dbg_mcucr,
      &dbg_demcr,
      &dbg_tpiuspp,
      &dbg_tpiuffc,
      &dbg_itmla,
      &dbg_itmtc,
      &dbg_itmtp,
    );
    clock_init(
      &pwr_cr1,
      &flash_acr,
      &rcc_apb1enr1,
      &rcc_cr,
      &rcc_cfgr,
      &rcc_cier,
      &rcc_pllcfgr,
      &rcc_bdcr,
    );
    peripheral_init(
      &rcc_ahb2enr,
      &gpiob_moder,
      &gpioc_moder,
      &gpiob_otyper,
      &gpioc_otyper,
      &gpiob_ospeedr,
      &gpioc_ospeedr,
    );
    VectorTable::config(move || {
      (
        NmiConfig { rcc_cifr, rcc_cicr },
        HardFaultConfig {},
        SysTickConfig {
          gpiob_cbsrr,
          gpioc_cbsrr,
        },
      )
    });
    exceptions_init(&stk_ctrl, &stk_load);
  }

  loop {
    util::wait_for_interrupt();
  }
}

#[cfg_attr(feature = "clippy", allow(too_many_arguments))]
unsafe fn clock_init<A, B, C, D, E, F, G, H>(
  pwr_cr1: &Reg<pwr::Cr1, A>,
  flash_acr: &Reg<flash::Acr, B>,
  rcc_apb1enr1: &Reg<rcc::Apb1enr1, C>,
  rcc_cr: &Reg<rcc::Cr, D>,
  rcc_cfgr: &Reg<rcc::Cfgr, E>,
  rcc_cier: &Reg<rcc::Cier, F>,
  rcc_pllcfgr: &Reg<rcc::Pllcfgr, G>,
  rcc_bdcr: &Reg<rcc::Bdcr, H>,
) {
  let pwr_cr1 = pwr_cr1.ptr();
  let flash_acr = flash_acr.ptr();
  let rcc_apb1enr1 = rcc_apb1enr1.ptr();
  let rcc_cr = rcc_cr.ptr();
  let rcc_cfgr = rcc_cfgr.ptr();
  let rcc_cier = rcc_cier.ptr();
  let rcc_pllcfgr = rcc_pllcfgr.ptr();
  let rcc_bdcr = rcc_bdcr.ptr();

  rcc_apb1enr1.modify(|reg| reg.power_enable(true));

  pwr_cr1.modify(|reg| reg.backup_domain_protection_disable(true));

  rcc_bdcr.modify(|reg| {
    reg
      .lse_enable(true)
      .lse_bypass(false)
      .rtc_source(BdcrRtcClock::Lse)
  });
  while !rcc_bdcr.read().lse_ready() {}
  rcc_bdcr.modify(|reg| reg.lse_css_enable(true));

  rcc_cier.modify(|reg| reg.lse_css_interrupt_enable(true));

  rcc_cr.modify(|reg| {
    reg
      .msi_pll_enable(true)
      .msi_range_selection()
      .msi_range(CrMsiRange::Range8Mhz)
  });

  rcc_pllcfgr.modify(|reg| {
    reg
      .pll_source(PllcfgrPllSource::Msi)
      .pllclk_enable(true)
      .pllclk_factor(PLLCLK_FACTOR)
      .pll_input_factor(PLL_INPUT_FACTOR)
      .pll_output_factor(PLL_OUTPUT_FACTOR)
  });

  rcc_cr.modify(|reg| reg.pll_enable(true));
  while !rcc_cr.read().pll_ready() {}

  flash_acr.modify(|reg| {
    reg
      .prefetch_enable(true)
      .instruction_cache_enable(true)
      .data_cache_enable(true)
      .latency(2)
  });

  rcc_cfgr.modify(|reg| reg.system_clock(CfgrSystemClock::Pll));
}

unsafe fn peripheral_init<A, B, C, D, E, F, G>(
  rcc_ahb2enr: &Reg<rcc::Ahb2enr, A>,
  gpiob_moder: &Reg<gpio::Moder<gpio::port::B>, B>,
  gpioc_moder: &Reg<gpio::Moder<gpio::port::C>, C>,
  gpiob_otyper: &Reg<gpio::Otyper<gpio::port::B>, D>,
  gpioc_otyper: &Reg<gpio::Otyper<gpio::port::C>, E>,
  gpiob_ospeedr: &Reg<gpio::Ospeedr<gpio::port::B>, F>,
  gpioc_ospeedr: &Reg<gpio::Ospeedr<gpio::port::C>, G>,
) {
  rcc_ahb2enr
    .ptr()
    .bits()
    .port_enable(Ahb2enrIop::B, true)
    .port_enable(Ahb2enrIop::C, true);
  gpiob_moder.ptr().modify(|reg| {
    reg
      .pin_config(ModerPin::P7, Mode::Output)
      .pin_config(ModerPin::P14, Mode::Output)
  });
  gpioc_moder
    .ptr()
    .modify(|reg| reg.pin_config(ModerPin::P7, Mode::Output));
  gpiob_otyper.ptr().modify(|reg| {
    reg
      .pin_config(OtyperPin::P7, Otype::PushPull)
      .pin_config(OtyperPin::P14, Otype::PushPull)
  });
  gpioc_otyper
    .ptr()
    .modify(|reg| reg.pin_config(OtyperPin::P7, Otype::PushPull));
  gpiob_ospeedr.ptr().modify(|reg| {
    reg
      .pin_config(OspeedrPin::P7, Ospeed::VeryHigh)
      .pin_config(OspeedrPin::P14, Ospeed::VeryHigh)
  });
  gpioc_ospeedr
    .ptr()
    .modify(|reg| reg.pin_config(OspeedrPin::P7, Ospeed::VeryHigh));
}

unsafe fn exceptions_init<A, B>(
  stk_ctrl: &Reg<stk::Ctrl, A>,
  stk_load: &Reg<stk::Load, B>,
) {
  stk_load.ptr().write(|reg| reg.value(SYS_TICK_SEC / 2048));
  stk_ctrl.ptr().modify(|reg| reg.enable(true).tick(true));
}
