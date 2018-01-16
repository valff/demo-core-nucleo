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
//! $ cargo drone flash --release
//! ```
//!
//! Listen to the ITM stream for connected device with the following command:
//!
//! ```sh
//! $ cargo drone server --itm
//! ```
//!
//! [Drone]: https://github.com/drone-os/drone
//! [NUCLEO-L496ZG-P]:
//! http://www.st.com/en/evaluation-tools/nucleo-l496zg-p.html

#![feature(alloc)]
#![feature(allocator_api)]
#![feature(allocator_internals)]
#![feature(compiler_builtins_lib)]
#![feature(conservative_impl_trait)]
#![feature(const_atomic_bool_new)]
#![feature(const_atomic_u32_new)]
#![feature(const_cell_new)]
#![feature(const_fn)]
#![feature(const_ptr_null_mut)]
#![feature(generators)]
#![feature(global_allocator)]
#![feature(integer_atomics)]
#![feature(naked_functions)]
#![feature(never_type)]
#![feature(prelude_import)]
#![feature(proc_macro)]
#![feature(slice_get_slice)]
#![default_lib_allocator]
#![no_std]
#![warn(missing_docs)]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![cfg_attr(feature = "clippy", allow(precedence, inline_always))]
#![cfg_attr(feature = "clippy", allow(diverging_sub_expression))]

extern crate alloc;
extern crate compiler_builtins;
#[macro_use]
extern crate drone_core;
#[macro_use]
extern crate drone_cortex_m;
extern crate futures;
#[cfg(test)]
#[macro_use]
extern crate test;

#[macro_use]
pub mod peripherals;

pub mod consts;
pub mod heap;
pub mod origin;
pub mod thread;

pub use heap::ALLOC;
pub use origin::origin;

#[prelude_import]
#[allow(unused_imports)]
use drone_cortex_m::prelude::*;
