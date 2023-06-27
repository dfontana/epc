use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, Duration, LocalResult, TimeZone, Utc};
use clap::ValueEnum;

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

impl Display for Precision {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let v = match self {
      Precision::Weeks => "weeks",
      Precision::Days => "days",
      Precision::Hours => "hours",
      Precision::Mins => "mins",
      Precision::Secs => "seconds",
      Precision::Millis => "milliseconds",
      Precision::Nanos => "nanoseconds",
    };
    write!(f, "{}", v)
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

  pub fn parse(&self, ts: i64) -> LocalResult<DateTime<Utc>> {
    match self {
      Precision::Millis => Utc.timestamp_millis_opt(ts),
      Precision::Nanos => LocalResult::Single(Utc.timestamp_nanos(ts)),
      _ => Utc.timestamp_opt(ts * self.seconds_per(), 0),
    }
  }

  /// Convert the given datetime into this precision, capping at
  /// seconds to prevent loss. See as_self_lossy for a lossy version.
  pub fn as_self<Tz: TimeZone>(&self, dt: &DateTime<Tz>) -> (i64, Self) {
    match self {
      Precision::Nanos => (dt.timestamp_nanos(), Precision::Nanos),
      Precision::Millis => (dt.timestamp_millis(), Precision::Millis),
      _ => (dt.timestamp(), Precision::Secs),
    }
  }

  /// Convert the given duration into this precision, losing precision
  /// for anything greater than seconds (truncating downwards)
  pub fn as_self_lossy(&self, dur: Duration) -> i64 {
    todo!()
  }

  /// The number of seconds in this precision tier. 0 if less than 1
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

  pub fn as_stamp<T>(&self, dt: &DateTime<T>) -> i64
  where
    T: TimeZone,
  {
    match self {
      Precision::Secs => dt.timestamp(),
      Precision::Millis => dt.timestamp_millis(),
      Precision::Nanos => dt.timestamp_nanos(),
      _ => dt.timestamp() / self.seconds_per(),
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
