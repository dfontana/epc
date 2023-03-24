use std::{
  cmp::Ordering,
  io::{self, Write},
  ops::{Add, Sub},
  str::FromStr,
};

use chrono::{
  format::{Item, StrftimeItems},
  DateTime, FixedOffset, Local, LocalResult, TimeZone, Utc,
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

  fn as_stamp<T>(&self, dt: DateTime<T>) -> i64
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

trait Handler {
  fn handle<W, E>(&self, output: W, error: E) -> Result<(), io::Error>
  where
    W: Write,
    E: Write;
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(args_conflicts_with_subcommands = true)]
struct Cli {
  #[command(subcommand)]
  commands: Option<Commands>,

  #[command(flatten)]
  current: CurrentArgs,
}

#[derive(Subcommand)]
enum Commands {
  /// (default) Get the current epoch time
  Current(CurrentArgs),
  /// Convert a list of epoch timestamps into date strings or vice versa
  Convert(ConvArgs),
  /// Get information on supported timezones
  Timezone(TzArgs),
  // TODO: Delta. Eg get diff of N time-likes and print human legible
}

#[derive(Args)]
struct TzArgs {}

impl Handler for TzArgs {
  fn handle<W, E>(&self, mut out: W, _err: E) -> Result<(), io::Error>
  where
    W: Write,
    E: Write,
  {
    TZ_VARIANTS
      .iter()
      .map(|f| writeln!(&mut out, "{}", f))
      .collect()
  }
}

#[derive(Clone)]
struct Format(String);

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
struct ConvArgs {
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

#[derive(Args)]
struct CurrentArgs {
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

impl Handler for CurrentArgs {
  fn handle<W, E>(&self, mut out: W, _err: E) -> Result<(), io::Error>
  where
    W: Write,
    E: Write,
  {
    // TODO: There has to be a better way to do this. For some reason though
    //       I can't figure out how to get the DTs into the same type
    match self.at_timezone {
      Some(LocalOrTz::TZ(tz)) => {
        let dt = Utc::now().with_timezone(&tz);
        if let Some(fmt) = &self.output_format {
          writeln!(&mut out, "{}", dt.format(&fmt.0))
        } else {
          writeln!(&mut out, "{}", self.precision.as_stamp(dt))
        }
      }
      Some(LocalOrTz::Local) => {
        let dt = Local::now();
        if let Some(fmt) = &self.output_format {
          writeln!(&mut out, "{}", dt.format(&fmt.0))
        } else {
          writeln!(&mut out, "{}", self.precision.as_stamp(dt))
        }
      }
      None => {
        let dt = Utc::now();
        if let Some(fmt) = &self.output_format {
          writeln!(&mut out, "{}", dt.format(&fmt.0))
        } else {
          writeln!(&mut out, "{}", self.precision.as_stamp(dt))
        }
      }
    }
  }
}

fn main() -> Result<(), io::Error> {
  let cli = Cli::parse();
  let output = io::stdout();
  let error = io::stderr();
  run(cli, output, error)
}

fn run<W, E>(cli: Cli, output: W, error: E) -> Result<(), io::Error>
where
  W: Write,
  E: Write,
{
  match cli.commands {
    Some(Commands::Timezone(tza)) => tza.handle(output, error),
    Some(Commands::Convert(conv)) => conv.handle(output, error),
    Some(Commands::Current(curr)) => curr.handle(output, error),
    None => cli.current.handle(output, error),
  }
}

#[cfg(test)]
mod test {
  use crate::{run, Cli};
  use clap::Parser;
  use indoc::indoc;

  fn run_test(cli_str: &str) -> (String, String) {
    let mut output = Vec::new();
    let mut error = Vec::new();
    let cli = Cli::try_parse_from(cli_str.split(' ')).expect("Could not parse args");
    run(cli, &mut output, &mut error).expect("Failed to run");
    let output = String::from_utf8(output).expect("Not UTF-8");
    let error = String::from_utf8(error).expect("Not UTF-8");
    (output, error)
  }

  #[test]
  fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert()
  }

  #[test]
  fn verify_stamp() {
    let (output, error) =
      run_test(" -p secs -t America/New_York 1679258022 1676258186 1679258186 -o dsc -f");
    assert_eq!("", error);
    assert_eq!(
      indoc! {"
        2023-03-19T16:36:26-0400
        2023-03-19T16:33:42-0400
        2023-02-12T22:16:26-0500
      "},
      output
    );
  }

  #[test]
  fn no_sort() {
    let (output, error) = run_test(" 1679258022 1676258187 1679258186");
    assert_eq!("", error);
    assert_eq!(
      indoc! {"
        1679258022
        1676258187
        1679258186
      "},
      output
    );
  }

  #[test]
  fn sort_asc() {
    let (output, error) = run_test(" 1679258022 1676258187 1679258186 -o asc");
    assert_eq!("", error);
    assert_eq!(
      indoc! {"
        1676258187
        1679258022
        1679258186
      "},
      output
    );
  }

  #[test]
  fn sort_dsc() {
    let (output, error) = run_test(" 1679258022 1676258187 1679258186 -o dsc");
    assert_eq!("", error);
    assert_eq!(
      indoc! {"
        1679258186
        1679258022
        1676258187
       "},
      output
    );
  }

  #[test]
  fn millis() {
    let (output, error) = run_test(" 1679661279000 1679661179000 1679661079000");
    assert_eq!("", error);
    assert_eq!(
      indoc! {"
        1679661279000
        1679661179000
        1679661079000
      "},
      output
    );
  }

  #[test]
  fn mixed_input() {
    let (output, error) = run_test(" -p secs 1679258022 2023-03-19T16:36:26-0400 1679258186");
    assert_eq!("", error);
    assert_eq!(
      indoc! {"
        1679258022
        1679258186
        1679258186
      "},
      output
    );
  }

  #[test]
  fn string_only() {
    let (output, error) =
      run_test(" 2023-03-19T16:36:26-0400 2023-03-19T16:33:42-0400 2023-02-12T22:16:26-0500");
    assert_eq!("", error);
    assert_eq!(
      indoc! {"
        1679258186000
        1679258022000
        1676258186000
      "},
      output
    );
  }
}
