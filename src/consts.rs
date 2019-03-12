//! Project constants.

/// MSI (multispeed internal) RC oscillator clock frequency.
pub const MSI_CLK: usize = 8_000_000;

/// Division factor for the main PLL input clock.
pub const PLL_INPUT_FACTOR: usize = 1;

/// Main PLL multiplication factor for VCO.
pub const PLL_OUTPUT_FACTOR: usize = 20;

/// Main PLL division factor for PLLCLK (system clock).
pub const PLLCLK_FACTOR: usize = 2;

/// Processor clock frequency.
pub const HCLK: usize =
  MSI_CLK / PLL_INPUT_FACTOR * PLL_OUTPUT_FACTOR / PLLCLK_FACTOR;

/// SysTick clocks in one second.
pub const SYS_TICK_SEC: usize = HCLK / 8;
