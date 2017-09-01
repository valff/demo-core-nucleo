//! Invoked on power up and warm reset.

use VectorTable;
use consts::SYS_TICK_SEC;
use drone::{itm, memory, util};
use drone::exception::ExceptionTable;
use drone::reg::{stk, AliasPointer, Delegate, Reg, Sreg, ValuePointer};
use drone::reg::gpio::{self, Mode, ModerPin, Ospeed, OspeedrPin, Otype,
                       OtyperPin};
use drone::reg::rcc::{self, Ahb2enrBits, Ahb2enrIop};
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
  let rcc_ahb2enr = Sreg::new();
  let rcc_cicr = Sreg::new();
  let rcc_cifr = Sreg::new();
  let gpio_moder = Sreg::new();
  let gpio_otyper = Sreg::new();
  let gpio_ospeedr = Sreg::new();
  let gpio_cbsrr = Sreg::new();

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
    peripheral_init(&rcc_ahb2enr, &gpio_moder, &gpio_otyper, &gpio_ospeedr);
    VectorTable::config(move || {
      (
        NmiConfig { rcc_cicr, rcc_cifr },
        HardFaultConfig {},
        SysTickConfig { gpio_cbsrr },
      )
    });
    exceptions_init(&stk_ctrl, &stk_load);
  }

  loop {
    util::wait_for_interrupt();
  }
}

unsafe fn peripheral_init<A, B, C, D>(
  rcc_ahb2enr: &Reg<rcc::Ahb2enr, A>,
  gpio_moder: &Reg<gpio::Moder<gpio::port::B>, B>,
  gpio_otyper: &Reg<gpio::Otyper<gpio::port::B>, C>,
  gpio_ospeedr: &Reg<gpio::Ospeedr<gpio::port::B>, D>,
) {
  rcc_ahb2enr.ptr().bits().port_enable(Ahb2enrIop::B, true);
  gpio_moder
    .ptr()
    .modify(|reg| reg.pin_config(ModerPin::P7, Mode::Output));
  gpio_otyper
    .ptr()
    .modify(|reg| reg.pin_config(OtyperPin::P7, Otype::PushPull));
  gpio_ospeedr
    .ptr()
    .modify(|reg| reg.pin_config(OspeedrPin::P7, Ospeed::VeryHigh));
}

unsafe fn exceptions_init<A, B>(
  stk_ctrl: &Reg<stk::Ctrl, A>,
  stk_load: &Reg<stk::Load, B>,
) {
  stk_load
    .ptr()
    .write(|reg| reg.value((SYS_TICK_SEC / 2048) >> 5));
  stk_ctrl.ptr().modify(|reg| reg.enable(true).tick(true));
}
