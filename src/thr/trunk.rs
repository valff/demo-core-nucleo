//! The reset handler.

use crate::{
    reg::Regs,
    thr::{self, Thrs},
};
use drone_cortex_m::{drv_sys_tick, drv_thr, reg::prelude::*, thr::prelude::*};
use drone_stm32_drv::gpio::GpioHead;
use drone_stm32_map::periph::{
    exti::periph_exti13,
    gpio::{
        periph_gpio_b14, periph_gpio_b7, periph_gpio_b_head, periph_gpio_c13, periph_gpio_c7,
        periph_gpio_c_head,
    },
};

/// The entry point.
#[inline(never)]
pub fn handler(reg: Regs) {
    let thr: Thrs = drv_thr!(reg).init(|scb_ccr| scb_ccr.set_div_0_trp());
    let lse = drv_lse!(reg);
    let msi = drv_msi!(reg);
    let pll = drv_pll!(reg);
    let flash = drv_flash!(reg);
    let sys_tick = drv_sys_tick!(reg, thr.sys_tick.to_attach());
    let gpio_b = GpioHead::new(periph_gpio_b_head!(reg));
    let gpio_b14 = periph_gpio_b14!(reg);
    let gpio_b7 = periph_gpio_b7!(reg);
    let gpio_c = GpioHead::new(periph_gpio_c_head!(reg));
    let gpio_c13 = periph_gpio_c13!(reg);
    let gpio_c7 = periph_gpio_c7!(reg);
    let exti13 = periph_exti13!(reg);

    let pwr_cr1 = reg.pwr_cr1.into_unsync();
    let mut rcc_cfgr = reg.rcc_cfgr.into_unsync();

    // Setup fault handlers
    thr.hard_fault.add_fn(|| panic!("Hard Fault"));

    // Disable backup domain write protection
    reg.rcc_apb1enr1.pwren.set_bit();
    pwr_cr1.dbp.set_bit_band();
    reg.rcc_apb1enr1.pwren.clear_bit();

    // Setup clocks
    lse.init();
    msi.init();
    pll.init();
    flash.init();
    rcc_cfgr.modify(|r| r.write_sw(0b11));

    // Setup interrupts
    thr.led.enable_batch(|r| {
        thr.led.enable(r);
        thr.button.enable(r);
    });
    thr.exti15_10.enable_int();

    // Configure push-button and LED pins.
    reg.rcc_apb2enr.modify(|r| r.set_syscfgen());
    exti13.syscfg_exticr_exti.write_bits(0b0010);
    exti13.exti_imr_im.set_bit();
    exti13.exti_rtsr_rt.set_bit();
    gpio_b.into_enabled();
    gpio_c.into_enabled();
    gpio_b7.gpio_moder_moder.modify(|r| {
        gpio_b7.gpio_moder_moder.write(r, 0b01);
        gpio_b14.gpio_moder_moder.write(r, 0b01);
    });
    gpio_c7.gpio_moder_moder.modify(|r| {
        gpio_c7.gpio_moder_moder.write(r, 0b01);
        gpio_c13.gpio_moder_moder.write(r, 0b00);
    });
    gpio_b7.gpio_otyper_ot.modify(|r| {
        gpio_b7.gpio_otyper_ot.clear(r);
        gpio_b14.gpio_otyper_ot.clear(r);
    });
    gpio_c7.gpio_otyper_ot.clear_bit();
    gpio_b7.gpio_ospeedr_ospeedr.modify(|r| {
        gpio_b7.gpio_ospeedr_ospeedr.write(r, 0b11);
        gpio_b14.gpio_ospeedr_ospeedr.write(r, 0b11);
    });
    gpio_c7.gpio_ospeedr_ospeedr.write_bits(0b11);
    gpio_c13.gpio_pupdr_pupdr.write_bits(0b10);

    // Setup push-button routine.
    thr.button.exec(thr::button::handler(thr::button::Input {
        exti13,
        exti15_10: thr.exti15_10.to_attach(),
        sys_tick: thr.sys_tick.to_attach(),
    }));
    // Setup LED blink routine.
    thr.led.exec(thr::led::handler(thr::led::Input {
        sys_tick,
        gpio_b14,
        gpio_b7,
        gpio_c7,
    }));

    // Do not return to the reset handler from interrupts.
    reg.scb_scr.modify(|r| r.set_sleeponexit());
}
