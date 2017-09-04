//! Project constants.

/// MSI (multispeed internal) RC oscillator clock frequency.
pub const MSI_CLK: u32 = 8_000_000;

/// Division factor for the main PLL and audio PLL (PLLSAI1 and PLLSAI2) input
/// clock.
pub const PLL_INPUT_FACTOR: u32 = 1;

/// Main PLL multiplication factor for VCO.
pub const PLL_OUTPUT_FACTOR: u32 = 20;

/// Main PLL division factor for PLLCLK (system clock).
pub const PLLCLK_FACTOR: u32 = 2;

/// Processor clock frequency.
pub const HCLK: u32 =
  MSI_CLK / PLL_INPUT_FACTOR * PLL_OUTPUT_FACTOR / PLLCLK_FACTOR;

/// SysTick clock frequency.
pub const SYS_TICK_SEC: u32 = HCLK / 8;
