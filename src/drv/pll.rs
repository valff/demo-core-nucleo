//! Phase-Locked Loop clock.

use crate::{
    consts::{PLLCLK_FACTOR, PLL_INPUT_FACTOR, PLL_OUTPUT_FACTOR},
    periph::pll::PllPeriph,
};
use drone_cortex_m::reg::prelude::*;

/// Acquires [`Pll`].
#[macro_export]
macro_rules! drv_pll {
    ($reg:ident) => {
        $crate::drv::pll::Pll::new(periph_pll!($reg))
    };
}

/// PLL driver.
pub struct Pll {
    periph: PllPeriph,
}

impl Pll {
    /// Creates a new [`Pll`].
    #[inline]
    pub fn new(periph: PllPeriph) -> Self {
        Self { periph }
    }

    /// Releases the peripheral.
    #[inline]
    pub fn free(self) -> PllPeriph {
        self.periph
    }

    /// Initializes PLL.
    pub fn init(&self) {
        self.periph.rcc_pllcfgr.store(|r| {
            r.write_pllsrc(0b01)
                .write_pllm(PLL_INPUT_FACTOR as u32 - 1)
                .write_plln(PLL_OUTPUT_FACTOR as u32)
                .write_pllr((PLLCLK_FACTOR as u32 >> 1) - 1)
                .set_pllren()
        });
        self.periph.rcc_cr_pllon.set_bit();
        while !self.periph.rcc_cr_pllrdy.read_bit() {}
    }
}
