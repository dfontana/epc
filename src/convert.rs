use std::{
  cmp::Ordering,
  io::{self, Write},
  ops::{Add, Sub},
  str::FromStr,
};

use chrono::{DateTime, FixedOffset};
use chrono_tz::Tz;
use clap::Args;
use itertools::Itertools;

use crate::types::{Format, Handler, Order, Precision};

#[derive(Args)]
pub struct ConvArgs {
  /// Mixture of Epoch timestamps in the given precision or date-time strings
  #[arg()]
  input: Vec<ConversionInput>,

  /// Convert to the given timezone. Omission will retain UTC. Accepts IANA names.
  #[arg(long, short='t', default_value_t = Tz::UTC)]
  at_timezone: Tz,

  /// What format to print the date strings in. Omitting will retain timestamps.
  ///
  /// Valid specifiers can be found at https://docs.rs/chrono/latest/chrono/format/strftime/index.html
  /// A reasonable default has been given, allowing you to pass -f alone
  #[arg(long, short = 'f', default_missing_value = "%Y-%m-%dT%H:%M:%S%z", require_equals=true, num_args=0..=1)]
  output_format: Option<Format>,

  /// What precision timestamps should be treated as
  #[arg(value_enum, long, short, default_value_t=Precision::MILLIS)]
  precision: Precision,

  /// When supplying multiple timestamps what order to print them in
  #[arg(value_enum, long, short)]
  order: Option<Order>,

  /// Subtract a human friendly duration to all times
  #[arg(long, short = 's', conflicts_with = "add")]
  sub: Option<humantime::Duration>,

  /// Add a human friendly duration to all times
  #[arg(long, short = 'a', conflicts_with = "sub")]
  add: Option<humantime::Duration>,
}

// TODO: Needs better error handling story. Trying to write to stderr
//       and silently continuing isn't great. Would be better to just raise
//       the exceptions
impl Handler for ConvArgs {
  fn handle<W, E>(&self, mut out: W, mut err: E) -> Result<(), io::Error>
  where
    W: Write,
    E: Write,
  {
    self
      .input
      .iter()
      // Extract as datetime
      .filter_map(|inp| {
        match inp.to_dt(&self.precision) {
          Ok(v) => Some(v),
          Err(e) => {
            // Swallow errors, there's not much more to be done
            writeln!(&mut err, "{}", e).unwrap_or(());
            None
          }
        }
      })
      // Convert to the given timezone
      .map(|dt: DateTime<FixedOffset>| dt.with_timezone(&self.at_timezone))
      .map(|dt| {
        // TODO: This isn't great, trying to shove nanos as the common denominator.
        //       Maybe we can make a better conversion method? Or perhaps add the
        //       Components?
        // TODO: Error handling here is lacklustre
        if let Some(dur) = self.add {
          match dur.as_nanos().try_into() {
            Ok(nanos) => dt.add(chrono::Duration::nanoseconds(nanos)),
            Err(_) => dt,
          }
        } else if let Some(dur) = self.sub {
          match dur.as_nanos().try_into() {
            Ok(nanos) => dt.sub(chrono::Duration::nanoseconds(nanos)),
            Err(_) => dt,
          }
        } else {
          dt
        }
      })
      // Apply ordering
      .sorted_by(|a, b| match self.order {
        Some(Order::DSC) => Ord::cmp(&a, &b).reverse(),
        Some(Order::ASC) => Ord::cmp(&a, &b),
        None => Ordering::Equal,
      })
      // Apply output formatting
      .map(|dt| {
        if let Some(fmt) = &self.output_format {
          writeln!(&mut out, "{}", dt.format(&fmt.0))
        } else {
          writeln!(&mut out, "{}", self.precision.as_stamp(dt))
        }
      })
      .collect()
  }
}

#[derive(Clone)]
enum ConversionInput {
  Stamp(i64),
  String(DateTime<FixedOffset>),
}

impl ConversionInput {
  fn to_dt(&self, precision: &Precision) -> Result<DateTime<FixedOffset>, String> {
    match self {
      ConversionInput::String(dt) => Ok(dt.clone()),
      ConversionInput::Stamp(ts) => precision
        .parse(*ts)
        .single()
        .map(|dt| dt.into())
        .ok_or_else(|| format!("Could not parse: {}", ts)),
    }
  }
}

impl FromStr for ConversionInput {
  type Err = String;

  fn from_str(arg: &str) -> Result<Self, Self::Err> {
    match arg.parse::<i64>() {
      Ok(ts) => Ok(ConversionInput::Stamp(ts)),
      Err(_) => match arg.parse::<DateTime<FixedOffset>>() {
        Ok(dt) => Ok(ConversionInput::String(dt)),
        Err(_) => Err(format!("Could not parse: {}", arg)),
      },
    }
  }
}
