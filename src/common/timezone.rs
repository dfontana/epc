use std::str::FromStr;

use chrono_tz::Tz;
use clap::Args;

#[derive(Clone)]
pub struct AutoTz(pub Tz);

impl FromStr for AutoTz {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let pstr = if s == "local" {
      iana_time_zone::get_timezone().map_err(|_| "Failed to lookup system timezone")?
    } else {
      s.to_string()
    };
    pstr
      .parse::<Tz>()
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
  pub fn get(&self) -> Tz {
    self.at_timezone.as_ref().map(|v| v.0).unwrap_or(Tz::UTC)
  }
}
