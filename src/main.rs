use chrono::{
  format::{Item, StrftimeItems},
  DateTime, FixedOffset, LocalResult, TimeZone, Utc,
};
use chrono_tz::{Tz, TZ_VARIANTS};
use clap::{Args, Parser, Subcommand, ValueEnum};
use itertools::Itertools;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Order {
  /// Ascending in time
  ASC,
  /// Descending in time
  DSC,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Precision {
  /// Seconds
  SECS,
  /// Milliseconds
  MILLIS,
  /// Nanoseconds
  NANOS,
}
impl Precision {
  fn parse(&self, ts: i64) -> LocalResult<DateTime<Utc>> {
    match self {
      Precision::SECS => Utc.timestamp_opt(ts, 0),
      Precision::MILLIS => Utc.timestamp_millis_opt(ts),
      Precision::NANOS => LocalResult::Single(Utc.timestamp_nanos(ts)),
    }
  }

  fn as_stamp(&self, dt: DateTime<Tz>) -> i64 {
    match self {
      Precision::SECS => dt.timestamp(),
      Precision::MILLIS => dt.timestamp_millis(),
      Precision::NANOS => dt.timestamp_nanos(),
    }
  }
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(args_conflicts_with_subcommands = true)]
struct Cli {
  #[command(subcommand)]
  commands: Option<Commands>,

  #[command(flatten)]
  from_stamp: FromStampArgs,
}

// TODO: Calc'ing. eg get the current time. Manipulate it by adding/subtracting.
//       manipulate a given time.

#[derive(Subcommand)]
enum Commands {
  /// (default) Convert a list of epoch timestamps into date strings
  FromStamps(FromStampArgs),
  /// Convert a list of date strings into epoch timestamps
  #[command(short_flag = 's')]
  FromStrings(FromStringArgs),
  /// Get information on supported timezones
  Timezone,
}

#[derive(Args)]
struct FromStampArgs {
  /// Epoch timestamps in the given precision
  #[arg(value_parser = clap::value_parser!(i64).range(0..))]
  timestamps: Vec<i64>,

  /// Convert to the given timezone. Omission will retain UTC. Accepts IANA names.
  #[arg(long, short, default_value_t = Tz::UTC)]
  at_timezone: Tz,

  /// What format to print the date strings in. Omitting will retain timestamps.
  ///
  /// Valid specifiers can be found at https://docs.rs/chrono/latest/chrono/format/strftime/index.html
  // TODO: Could use a default format arg exclusive with this for: %Y-%m-%dT%H:%M:%S%z
  #[arg(long, short='f', value_parser = parse_format)]
  output_format: Option<String>,

  /// What precision timestamps should be treated as
  #[arg(value_enum, long, short, default_value_t=Precision::MILLIS)]
  precision: Precision,

  /// When supplying multiple timestamps what order to print them in
  #[arg(value_enum, long, short, default_value_t = Order::DSC)]
  order: Order,
}

#[derive(Args)]
struct FromStringArgs {
  /// Date strings in Fixed Offset format: 2014-11-28T21:00:09+09:00
  #[arg()]
  timestrings: Vec<DateTime<FixedOffset>>,

  /// Convert to the given timezone. Omission will use UTC. Accepts IANA names.
  #[arg(long, short, default_value_t = Tz::UTC)]
  at_timezone: Tz,

  /// What format to print the date strings in. Omitting will retain timestamps.
  ///
  /// Valid specifiers can be found at https://docs.rs/chrono/latest/chrono/format/strftime/index.html
  #[arg(long, short, value_parser = parse_format)]
  output_format: Option<String>,

  /// What precision timestamps should be treated as
  #[arg(value_enum, long, short, default_value_t=Precision::MILLIS)]
  precision: Precision,

  /// When supplying multiple timestamps what order to print them in
  #[arg(value_enum, long, default_value_t = Order::DSC)]
  order: Order,
}

fn parse_format(s: &str) -> Result<String, String> {
  if StrftimeItems::new(s).any(|v| matches!(v, Item::Error)) {
    Err("contains unknown specifier".into())
  } else {
    Ok(s.into())
  }
}

fn main() {
  let cli = Cli::parse();
  match cli.commands {
    Some(Commands::Timezone) => {
      handle_timezone();
    }
    Some(Commands::FromStamps(from_stamp)) => {
      handle_from_stamps(from_stamp);
    }
    Some(Commands::FromStrings(from_string)) => {
      handle_from_strings(from_string);
    }
    None => handle_from_stamps(cli.from_stamp),
  }
}

struct InternalConversion {
  at_timezone: Tz,
  output_format: Option<String>,
  precision: Precision,
  order: Order,
}

impl From<&FromStampArgs> for InternalConversion {
  fn from(args: &FromStampArgs) -> Self {
    InternalConversion {
      at_timezone: args.at_timezone,
      output_format: args.output_format.clone(),
      precision: args.precision,
      order: args.order,
    }
  }
}

impl From<&FromStringArgs> for InternalConversion {
  fn from(args: &FromStringArgs) -> Self {
    InternalConversion {
      at_timezone: args.at_timezone,
      output_format: args.output_format.clone(),
      precision: args.precision,
      order: args.order,
    }
  }
}

impl InternalConversion {
  fn apply(&self, iter: impl IntoIterator<Item = DateTime<FixedOffset>>) {
    // Convert to the given timezone
    iter
      .into_iter()
      .map(|dt| dt.with_timezone(&self.at_timezone))
      // Apply ordering
      .sorted_by(|a, b| {
        let mut ord = Ord::cmp(&a, &b);
        if self.order == Order::DSC {
          ord = ord.reverse();
        }
        ord
      })
      // Apply output formatting
      .map(|dt| {
        if let Some(fmt) = &self.output_format {
          format!("{}", dt.format(fmt))
        } else {
          format!("{}", self.precision.as_stamp(dt))
        }
      })
      // Output
      .for_each(|fstr| println!("{}", fstr))
  }
}

fn handle_timezone() {
  TZ_VARIANTS.iter().for_each(|f| println!("{}", f))
}

fn handle_from_stamps(args: FromStampArgs) {
  InternalConversion::from(&args).apply(
    args
      .timestamps
      .iter()
      .map(|ts| (ts, args.precision.parse(*ts)))
      .filter_map(|(ts, dtr)| {
        if let Some(dt) = dtr.single() {
          Some(dt)
        } else {
          println!("Could not interpret {}", ts);
          None
        }
      })
      .map(|dt| dt.into()),
  )
}

fn handle_from_strings(args: FromStringArgs) {
  InternalConversion::from(&args).apply(args.timestrings)
}

#[test]
fn verify_cli() {
  use clap::CommandFactory;
  Cli::command().debug_assert()
}
