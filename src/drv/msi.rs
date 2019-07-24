//! Multispeed Internal RC oscillator clock.

use crate::periph::msi::MsiPeriph;
use drone_cortex_m::reg::prelude::*;

/// Acquires [`Msi`].
#[macro_export]
macro_rules! drv_msi {
    ($reg:ident) => {
        $crate::drv::msi::Msi::new(periph_msi!($reg))
    };
}

/// MSI driver.
pub struct Msi {
    periph: MsiPeriph,
}

impl Msi {
    /// Creates a new [`Msi`].
    #[inline]
    pub fn new(periph: MsiPeriph) -> Self {
        Self { periph }
    }

    /// Releases the peripheral.
    #[inline]
    pub fn free(self) -> MsiPeriph {
        self.periph
    }

    /// Initializes MSI.
    pub fn init(&self) {
        self.periph.rcc_cr_msipllen.modify(|r| {
            self.periph.rcc_cr_msipllen.set(r);
            self.periph.rcc_cr_msirange.write(r, 0b0111);
            self.periph.rcc_cr_msirgsel.set(r);
        });
    }
}
