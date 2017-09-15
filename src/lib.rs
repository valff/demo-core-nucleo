//! Example blinking program for [NUCLEO-L496ZG-P][nucleo-l496zg-p] board using
//! [Drone][drone] RTOS.
//!
//! [drone]: https://github.com/valff/drone
//! [nucleo-l496zg-p]:
//! http://www.st.com/en/evaluation-tools/nucleo-l496zg-p.html
#![feature(compiler_builtins_lib)]
#![feature(const_fn)]
#![feature(generators)]
#![feature(naked_functions)]
#![no_std]
#![warn(missing_docs)]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![cfg_attr(feature = "clippy", allow(precedence, doc_markdown))]

extern crate compiler_builtins;
extern crate drone;
#[macro_use]
extern crate drone_stm32;
#[cfg(test)]
#[macro_use]
extern crate test;

pub use reset::main;
pub use vtable::VectorTable;

pub mod consts;
pub mod vtable;
pub mod reset;
