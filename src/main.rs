use chrono::{DateTime, FixedOffset, TimeZone, Utc};
use chrono_tz::Tz;
use clap::{Args, Parser, Subcommand, ValueEnum};

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

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(args_conflicts_with_subcommands = true)]
struct Cli {
  #[command(subcommand)]
  convert: Option<Commands>,

  #[command(flatten)]
  from_stamp: FromStampArgs,
}

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
  #[arg(value_parser = clap::value_parser!(i32).range(0..))]
  timestamps: Vec<i32>,

  /// Convert to the given timezone. Accepts IANA names.
  #[arg(long, short)]
  at_timezone: Option<Tz>,

  /// What format to print the date strings in
  ///
  /// Valid specifiers can be found at https://docs.rs/chrono/latest/chrono/format/strftime/index.html
  #[arg(long, short, default_value = "%FT%T%:z")]
  format: Option<String>,

  /// What precision timestamps are
  #[arg(value_enum, long, short, default_value_t=Precision::MILLIS)]
  precision: Precision,

  /// When supplying multiple timestamps what order to print them in
  #[arg(value_enum, long, default_value_t = Order::DSC)]
  order: Order,
}

#[derive(Args)]
struct FromStringArgs {
  /// Date strings in Fixed Offset format: 2014-11-28T21:00:09+09:00
  #[arg()]
  timestrings: Vec<DateTime<FixedOffset>>,

  /// Convert to the given timezone. Accepts IANA names.
  #[arg(long, short)]
  at_timezone: Option<Tz>,

  /// What precision to interpret timestamps as
  #[arg(value_enum, long, short, default_value_t=Precision::MILLIS)]
  precision: Precision,

  /// When supplying multiple timestamps what order to print them in
  #[arg(value_enum, long, default_value_t = Order::DSC)]
  order: Order,
}

fn main() {
  let cli = Cli::parse();

  Utc.timestamp_opt(0, 0);
  Utc.timestamp_millis_opt(0);
  Utc.timestamp_nanos(0);
}

#[test]
fn verify_cli() {
  use clap::CommandFactory;
  Cli::command().debug_assert()
}
