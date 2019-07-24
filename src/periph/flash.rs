//! Embedded Flash memory.

use crate::reg;
use drone_core::periph;

periph::one! {
    /// Acquires Flash.
    pub macro periph_flash;

    /// Flash.
    pub struct FlashPeriph;

    reg; periph::flash;

    FLASH {
        ACR;
    }
}
