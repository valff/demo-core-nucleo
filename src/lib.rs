//! Example blinking program for *STM32L4*-series MCU.
#![feature(asm)]
#![feature(compiler_builtins_lib)]
#![feature(const_fn)]
#![feature(drop_types_in_const)]
#![feature(generators)]
#![feature(generator_trait)]
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
