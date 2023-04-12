use std::ops::Neg;

use chrono::DateTime;
use chrono_tz::Tz;
use clap::Args;

use crate::hduration::HDuration;

#[derive(Args)]
pub struct CalcArgs {
  /// Add a human friendly duration to all times (can be negative)
  #[arg(long, short = 'a', allow_hyphen_values = true)]
  add: Option<HDuration>,
}

impl CalcArgs {
  pub fn eval(&self, dt: DateTime<Tz>) -> Result<DateTime<Tz>, String> {
    if let Some(dur) = &self.add {
      chrono::Duration::from_std(dur.inner)
        .map(|d| if dur.negative { d.neg() } else { d })
        .map_err(|e| format!("{}", e))
        .map(|d| dt + d)
    } else {
      Ok(dt)
    }
  }
}
