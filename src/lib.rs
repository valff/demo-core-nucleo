//! Example blinking program for [NUCLEO-L496ZG-P] board based on [Drone].
//!
//! # Effects
//!
//! * Smooth blinking with the all three user LEDs.
//! * Responding to the on-board button.
//! * Running MCU at the full speed (80 MHz).
//! * Using the on-board LSE for MSI auto-calibration.
//! * Printing messages through ITM.
//!
//! # Usage
//!
//! Flash the board with the following command:
//!
//! ```sh
//! $ RUSTC_WRAPPER=./rustc-wrapper.sh cargo drone flash --release
//! ```
//!
//! Listen to the ITM stream for connected device with the following command:
//!
//! ```sh
//! $ cargo drone server --itm
//! ```
//!
//! # Development
//!
//! Check:
//!
//! ```sh
//! $ RUSTC_WRAPPER=./clippy-wrapper.sh xargo check \
//!   --target "thumbv7em-none-eabihf"
//! ```
//!
//! Test:
//!
//! ```sh
//! $ RUSTC_WRAPPER=./rustc-wrapper.sh cargo drone test
//! ```
//!
//! Readme update:
//!
//! ```sh
//! $ cargo readme -o README.md
//! ```
//!
//! [Drone]: https://github.com/drone-os/drone
//! [NUCLEO-L496ZG-P]:
//! http://www.st.com/en/evaluation-tools/nucleo-l496zg-p.html

#![feature(alloc)]
#![feature(allocator_api)]
#![feature(allocator_internals)]
#![feature(compiler_builtins_lib)]
#![feature(const_fn)]
#![feature(generators)]
#![feature(integer_atomics)]
#![feature(naked_functions)]
#![feature(never_type)]
#![feature(prelude_import)]
#![feature(tool_lints)]
#![default_lib_allocator]
#![no_std]
#![warn(missing_docs)]
#![allow(clippy::precedence, clippy::inline_always)]
#![allow(clippy::diverging_sub_expression)]

extern crate alloc;
extern crate rlibc;
#[macro_use]
extern crate drone_core;
#[macro_use]
extern crate drone_stm32 as drone_plat;
extern crate futures;
#[cfg(test)]
#[macro_use]
extern crate test;

#[macro_use]
pub mod drv;

pub mod consts;
pub mod heap;
pub mod sv;
pub mod thr;
pub mod trunk;

pub use trunk::trunk;

#[prelude_import]
#[allow(unused_imports)]
use drone_plat::prelude::*;

/// The global allocator.
#[global_allocator]
pub static mut HEAP: heap::Heap = heap::Heap::new();
