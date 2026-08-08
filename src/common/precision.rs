use std::str::FromStr;

use clap::ValueEnum;
use jiff::{Timestamp, Zoned};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Precision {
  /// Weeks
  Weeks,
  /// Days
  Days,
  /// Hours
  Hours,
  /// Minutes,
  Mins,
  /// Seconds
  Secs,
  /// Milliseconds
  Millis,
  /// Nanoseconds
  Nanos,
}

impl FromStr for Precision {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let p = match s {
      "w" | "weeks" => Precision::Weeks,
      "d" | "days" => Precision::Days,
      "h" | "hours" => Precision::Hours,
      "m" | "mins" => Precision::Mins,
      "s" | "secs" => Precision::Secs,
      "ms" | "millis" => Precision::Millis,
      "ns" | "nanos" => Precision::Nanos,
      _ => return Err(format!("Unknown precision: {}", s)),
    };
    Ok(p)
  }
}

impl Precision {
  pub fn try_downcast(&self) -> Option<Self> {
    let p = match self {
      Precision::Weeks => Precision::Days,
      Precision::Days => Precision::Hours,
      Precision::Hours => Precision::Mins,
      Precision::Mins => Precision::Secs,
      Precision::Secs => Precision::Millis,
      Precision::Millis => Precision::Nanos,
      Precision::Nanos => return None,
    };
    Some(p)
  }

  pub fn parse(&self, ts: i64) -> Result<Timestamp, String> {
    match self {
      Precision::Millis => Timestamp::from_millisecond(ts).map_err(|e| e.to_string()),
      Precision::Nanos => Timestamp::from_nanosecond(ts.into()).map_err(|e| e.to_string()),
      _ => ts
        .checked_mul(self.seconds_per())
        .ok_or_else(|| format!("Could not parse: {}", ts))
        .and_then(|seconds| Timestamp::from_second(seconds).map_err(|e| e.to_string())),
    }
  }

  pub fn seconds_per(&self) -> i64 {
    match self {
      Precision::Weeks => 7 * self.try_downcast().map(|p| p.seconds_per()).unwrap_or(0),
      Precision::Days => 24 * self.try_downcast().map(|p| p.seconds_per()).unwrap_or(0),
      Precision::Hours | Precision::Mins => {
        60 * self.try_downcast().map(|p| p.seconds_per()).unwrap_or(0)
      }
      Precision::Secs => 1,
      _ => 0,
    }
  }

  pub fn as_stamp(&self, dt: &Zoned) -> i128 {
    let timestamp = dt.timestamp();
    match self {
      Precision::Secs => timestamp.as_second().into(),
      Precision::Millis => timestamp.as_millisecond().into(),
      Precision::Nanos => timestamp.as_nanosecond(),
      _ => (timestamp.as_second() / self.seconds_per()).into(),
    }
  }
}

#[cfg(test)]
mod test {
  use rstest::*;

  use super::Precision;

  #[rstest]
  #[case(Precision::Millis, 0)]
  #[case(Precision::Nanos, 0)]
  #[case(Precision::Secs, 1)]
  #[case(Precision::Mins, 60)]
  #[case(Precision::Hours, 3600)]
  #[case(Precision::Days, 86400)]
  #[case(Precision::Weeks, 604800)]
  fn seconds_per(#[case] pre: Precision, #[case] exp: i64) {
    assert_eq!(pre.seconds_per(), exp)
  }
}
