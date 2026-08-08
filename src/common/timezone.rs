use std::str::FromStr;

use clap::Args;
use jiff::tz::TimeZone;

#[derive(Clone)]
pub struct AutoTz(pub TimeZone);

impl FromStr for AutoTz {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if s == "local" {
      return Ok(AutoTz(TimeZone::system()));
    }
    TimeZone::get(s)
      .map(AutoTz)
      .map_err(|_| format!("{} is not a known timezone", s))
  }
}

#[derive(Args)]
pub struct AtTimezoneArgs {
  /// Convert to the given timezone. Omission will retain UTC. Accepts IANA names.
  /// passing -t alone will use the system local timezone
  #[arg(long, short='t', default_missing_value="local", require_equals=true, num_args=0..=1)]
  at_timezone: Option<AutoTz>,
}

impl AtTimezoneArgs {
  pub fn get(&self) -> TimeZone {
    self
      .at_timezone
      .as_ref()
      .map(|v| v.0.clone())
      .unwrap_or(TimeZone::UTC)
  }
}
