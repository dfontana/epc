use std::{
  fmt::Display,
  io::{self, Write},
};

use chrono::{DateTime, TimeZone, Utc};
use chrono_tz::Tz;
use clap::Args;

use crate::types::{AutoTz, Format, Handler, Precision};

#[derive(Args)]
pub struct CurrentArgs {
  /// Convert to the given timezone. Omission will retain UTC. Accepts IANA names.
  /// passing -t alone will use the system local timezone
  #[arg(long, short='t', default_missing_value="local", require_equals=true, num_args=0..=1)]
  at_timezone: Option<AutoTz>,

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
      writeln!(&mut out, "{}", self.precision.as_stamp(&dt))
    }
  }
}

impl Handler for CurrentArgs {
  fn handle<W, E>(&self, out: W, _err: E) -> Result<(), io::Error>
  where
    W: Write,
    E: Write,
  {
    let tz = self.at_timezone.as_ref().map(|v| v.0).unwrap_or(Tz::UTC);
    self.format(out, Utc::now().with_timezone(&tz))
  }
}
