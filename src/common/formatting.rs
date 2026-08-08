use std::str::FromStr;

use clap::Args;
use jiff::{fmt::strtime, Zoned};

use super::Precision;

const DEFAULT_FORMAT_SENTINEL: &str = "__epc_default_output_format__";
const DEFAULT_OUTPUT_FORMAT: &str = "%Y-%m-%dT%H:%M:%S%z";
const DEFAULT_UTC_OUTPUT_FORMAT: &str = "%Y-%m-%dT%H:%M:%SZ";

#[derive(Clone)]
enum Format {
  Default,
  Custom(String),
}

impl FromStr for Format {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if s == DEFAULT_FORMAT_SENTINEL {
      return Ok(Format::Default);
    }

    let format = s.replace("%+", "%Y-%m-%dT%H:%M:%S%.f%:z");
    strtime::format(&format, &Zoned::UNIX_EPOCH)
      .map(|_| Format::Custom(format))
      .map_err(|_| "contains unknown specifier".into())
  }
}

#[derive(Args)]
pub struct FormatArgs {
  /// What format to print the date strings in. Omitting will retain timestamps.
  ///
  /// Valid specifiers can be found at https://docs.rs/jiff/latest/jiff/fmt/strtime/index.html
  /// A reasonable default has been given, allowing you to pass -f alone
  #[arg(long, short = 'f', default_missing_value = DEFAULT_FORMAT_SENTINEL, require_equals=true, num_args=0..=1)]
  output_format: Option<Format>,

  /// What precision timestamps should be treated as
  #[arg(value_enum, long, short, default_value_t=Precision::Millis)]
  pub precision: Precision,
}

impl FormatArgs {
  pub fn format(&self, dt: &Zoned) -> String {
    match &self.output_format {
      Some(Format::Default) if dt.offset().is_zero() => {
        dt.strftime(DEFAULT_UTC_OUTPUT_FORMAT).to_string()
      }
      Some(Format::Default) => dt.strftime(DEFAULT_OUTPUT_FORMAT).to_string(),
      Some(Format::Custom(fmt)) => dt.strftime(fmt).to_string(),
      None => self.precision.as_stamp(dt).to_string(),
    }
  }
}
