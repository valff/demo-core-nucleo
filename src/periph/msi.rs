//! Multispeed Internal RC oscillator clock.

use crate::reg;
use drone_core::periph;

periph::one! {
    /// Acquires MSI.
    pub macro periph_msi;

    /// MSI.
    pub struct MsiPeriph;

    reg; periph::msi;

    RCC {
        CR {
            MSIPLLEN;
            MSIRANGE;
            MSIRGSEL;
        }
    }
}
