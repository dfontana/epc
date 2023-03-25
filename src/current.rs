use std::{
  fmt::Display,
  io::{self, Write},
  str::FromStr,
};

use chrono::{DateTime, Local, TimeZone, Utc};
use chrono_tz::Tz;
use clap::Args;

use crate::types::{Format, Handler, Precision};

#[derive(Args)]
pub struct CurrentArgs {
  /// Convert to the given timezone. Omission will retain UTC. Accepts IANA names.
  /// passing -t alone will use the system local timezone
  #[arg(long, short='t', default_missing_value="local", require_equals=true, num_args=0..=1)]
  at_timezone: Option<LocalOrTz>,

  /// What format to print the date strings in. Omitting will retain timestamps.
  ///
  /// Valid specifiers can be found at https://docs.rs/chrono/latest/chrono/format/strftime/index.html
  /// A reasonable default has been given, allowing you to pass -f alone
  #[arg(long, short = 'f', default_missing_value = "%Y-%m-%dT%H:%M:%S%z", require_equals=true, num_args=0..=1)]
  output_format: Option<Format>,

  /// What precision timestamps should be treated as
  #[arg(value_enum, long, short, default_value_t=Precision::MILLIS, conflicts_with="output_format")]
  precision: Precision,
}

impl CurrentArgs {
  fn format<T: TimeZone>(&self, mut out: impl Write, dt: DateTime<T>) -> Result<(), io::Error>
  where
    T::Offset: Display,
  {
    if let Some(fmt) = &self.output_format {
      writeln!(&mut out, "{}", dt.format(&fmt.0))
    } else {
      writeln!(&mut out, "{}", self.precision.as_stamp(dt))
    }
  }
}

impl Handler for CurrentArgs {
  fn handle<W, E>(&self, out: W, _err: E) -> Result<(), io::Error>
  where
    W: Write,
    E: Write,
  {
    match self.at_timezone {
      Some(LocalOrTz::TZ(tz)) => self.format(out, Utc::now().with_timezone(&tz)),
      Some(LocalOrTz::Local) => self.format(out, Local::now()),
      None => self.format(out, Utc::now()),
    }
  }
}

#[derive(Clone)]
enum LocalOrTz {
  TZ(Tz),
  Local,
}

impl FromStr for LocalOrTz {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if s == "local" {
      Ok(LocalOrTz::Local)
    } else {
      s.parse::<Tz>()
        .map(|t| LocalOrTz::TZ(t))
        .map_err(|_| format!("{} is not a known timezone", s))
    }
  }
}
