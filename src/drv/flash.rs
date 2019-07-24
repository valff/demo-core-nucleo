//! Embedded Flash memory.

use crate::periph::flash::FlashPeriph;
use drone_cortex_m::reg::prelude::*;

/// Acquires [`Flash`].
#[macro_export]
macro_rules! drv_flash {
    ($reg:ident) => {
        $crate::drv::flash::Flash::new(periph_flash!($reg))
    };
}

/// Flash driver.
pub struct Flash {
    periph: FlashPeriph,
}

impl Flash {
    /// Creates a new [`Flash`].
    #[inline]
    pub fn new(periph: FlashPeriph) -> Self {
        Self { periph }
    }

    /// Releases the peripheral.
    #[inline]
    pub fn free(self) -> FlashPeriph {
        self.periph
    }

    /// Initializes flash.
    pub fn init(&self) {
        self.periph
            .flash_acr
            .store(|r| r.set_prften().set_icen().set_dcen().write_latency(2));
    }
}
