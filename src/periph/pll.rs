//! Phase-Locked Loop clock.

use crate::reg;
use drone_core::periph;

periph::one! {
    /// Acquires PLL.
    pub macro periph_pll;

    /// PLL.
    pub struct PllPeriph;

    reg; periph::pll;

    RCC {
        CR {
            PLLON;
            PLLRDY;
        }
        PLLCFGR;
    }
}

periph::one! {
    /// Acquires PLLSAI1.
    pub macro periph_pllsai1;

    /// PLLSAI1.
    pub struct Pllsai1Periph;

    reg; periph::pll;

    RCC {
        CICR {
            PLLSAI1RDYC;
        }
        CIER {
            PLLSAI1RDYIE;
        }
        CIFR {
            PLLSAI1RDYF;
        }
        CR {
            PLLSAI1ON;
        }
        PLLSAI1CFGR;
    }
}
