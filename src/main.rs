use chrono::{DateTime, FixedOffset, LocalResult, TimeZone, Utc};
use chrono_tz::{Tz, TZ_VARIANTS};
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
  commands: Option<Commands>,

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
  #[arg(value_parser = clap::value_parser!(i64).range(0..))]
  timestamps: Vec<i64>,

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

fn handle_timezone() {
  TZ_VARIANTS.iter().for_each(|f| println!("{}", f))
}
fn handle_from_stamps(args: FromStampArgs) {
  let _parse = match args.precision {
    Precision::SECS => |i: i64| Utc.timestamp_opt(i, 0),
    Precision::MILLIS => |i: i64| Utc.timestamp_millis_opt(i),
    Precision::NANOS => |i: i64| LocalResult::Single(Utc.timestamp_nanos(i)),
  };
}
fn handle_from_strings(_args: FromStringArgs) {}

#[test]
fn verify_cli() {
  use clap::CommandFactory;
  Cli::command().debug_assert()
}
