use std::{fmt::Display, str::FromStr};

use chrono::{
  format::{Item, StrftimeItems},
  DateTime, TimeZone,
};
use clap::Args;

use super::Precision;

#[derive(Clone)]
struct Format(pub String);
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

#[derive(Args)]
pub struct FormatArgs {
  /// What format to print the date strings in. Omitting will retain timestamps.
  ///
  /// Valid specifiers can be found at https://docs.rs/chrono/latest/chrono/format/strftime/index.html
  /// A reasonable default has been given, allowing you to pass -f alone
  #[arg(long, short = 'f', default_missing_value = "%Y-%m-%dT%H:%M:%S%z", require_equals=true, num_args=0..=1)]
  output_format: Option<Format>,

  /// What precision timestamps should be treated as
  #[arg(value_enum, long, short, default_value_t=Precision::Millis)]
  pub precision: Precision,
}

impl FormatArgs {
  pub fn format<T: TimeZone>(&self, dt: &DateTime<T>) -> String
  where
    T::Offset: Display,
  {
    match &self.output_format {
      Some(fmt) => dt.format(&fmt.0).to_string(),
      None => self.precision.as_stamp(dt).to_string(),
    }
  }
}
