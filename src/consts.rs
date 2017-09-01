//! Project constants.

/// High-Speed External clock frequency.
pub const HSE_CLK: u32 = 8_000_000;

/// PLL multiplication factor.
pub const PLL_MULT: u32 = 9;

/// Processor clock frequency.
pub const HCLK: u32 = HSE_CLK * PLL_MULT;

/// SysTick clock frequency.
pub const SYS_TICK_SEC: u32 = HCLK / 8;
