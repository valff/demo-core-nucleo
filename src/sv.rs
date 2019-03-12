//! Supervisor.

use drone_cortex_m::sv;

sv! {
  /// The supervisor.
  pub struct Sv;
  /// Supervisor services.
  pub static SERVICES;
}
