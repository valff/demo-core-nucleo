//! Supervisor.

use drone_plat::sv;

sv! {
  /// The supervisor.
  pub struct Sv;
  /// Supervisor services.
  pub static SERVICES;
}
