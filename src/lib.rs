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
//! $ scripts/flash.sh
//! ```
//!
//! Listen to the ITM stream for connected device with the following command:
//!
//! ```sh
//! $ scripts/swo.sh
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
#![feature(futures_api)]
#![feature(generators)]
#![feature(integer_atomics)]
#![feature(naked_functions)]
#![feature(never_type)]
#![feature(nll)]
#![feature(prelude_import)]
#![feature(proc_macro_hygiene)]
#![default_lib_allocator]
#![no_std]
#![deny(bare_trait_objects)]
#![deny(elided_lifetimes_in_paths)]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(
  clippy::cast_possible_truncation,
  clippy::doc_markdown,
  clippy::enum_glob_use,
  clippy::precedence,
  clippy::similar_names
)]

#[macro_use]
pub mod periph;
#[macro_use]
pub mod drv;

pub mod consts;
pub mod heap;
pub mod reg;
pub mod sv;
pub mod thr;

#[prelude_import]
#[allow(unused_imports)]
use drone_cortex_m::prelude::*;

/// The global allocator.
#[global_allocator]
pub static mut HEAP: heap::Heap = heap::Heap::new();
