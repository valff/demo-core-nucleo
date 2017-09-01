//! Exception that occurs because of an error during exception processing, or
//! because an exception cannot be managed by any other exception mechanism.

use drone::exception::Exception;

static mut HARD_FAULT: HardFault = HardFault {};

/// The exception routine data.
pub struct HardFault {}

/// The exception configuration data.
pub struct HardFaultConfig {}

/// The exception handler.
pub extern "C" fn handler() {
  unsafe { HARD_FAULT.run() }
}

impl Exception for HardFault {
  type Config = HardFaultConfig;

  unsafe fn config(_config: HardFaultConfig) {}

  fn run(&mut self) {
    panic!("Hard Fault");
  }
}
