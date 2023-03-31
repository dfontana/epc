use std::{
  io::{self, Write},
  str::FromStr,
};

use chrono::{
  format::{Item, StrftimeItems},
  DateTime, LocalResult, TimeZone, Utc,
};
use chrono_tz::Tz;
use clap::ValueEnum;

pub trait Handler {
  fn handle<W, E>(&self, output: W, error: E) -> Result<(), io::Error>
  where
    W: Write,
    E: Write;
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Order {
  /// Ascending in time
  ASC,
  /// Descending in time
  DSC,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Precision {
  /// Seconds
  SECS,
  /// Milliseconds
  MILLIS,
  /// Nanoseconds
  NANOS,
}

#[derive(Clone)]
pub struct Format(pub String);

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
      .map(|t| AutoTz(t))
      .map_err(|_| format!("{} is not a known timezone", s))
  }
}

impl Precision {
  pub fn parse(&self, ts: i64) -> LocalResult<DateTime<Utc>> {
    match self {
      Precision::SECS => Utc.timestamp_opt(ts, 0),
      Precision::MILLIS => Utc.timestamp_millis_opt(ts),
      Precision::NANOS => LocalResult::Single(Utc.timestamp_nanos(ts)),
    }
  }

  pub fn as_stamp<T>(&self, dt: &DateTime<T>) -> i64
  where
    T: TimeZone,
  {
    match self {
      Precision::SECS => dt.timestamp(),
      Precision::MILLIS => dt.timestamp_millis(),
      Precision::NANOS => dt.timestamp_nanos(),
    }
  }
}

impl FromStr for Format {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if StrftimeItems::new(s).any(|v| matches!(v, Item::Error)) {
      Err("contains unknown specifier".into())
    } else {
      Ok(Format(s.into()))
    }
  }
}
