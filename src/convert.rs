use std::{
  cmp::Ordering,
  io::{self, Write},
  str::FromStr,
};

use chrono::{DateTime, FixedOffset};
use clap::{Args, ValueEnum};

use crate::{
  common::{AtTimezoneArgs, CalcArgs, FormatArgs, Precision},
  Handler,
};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Order {
  /// Ascending in time
  Asc,
  /// Descending in time
  Dsc,
}

#[derive(Args)]
pub struct ConvArgs {
  #[command(flatten)]
  timezone: AtTimezoneArgs,

  #[command(flatten)]
  format: FormatArgs,

  #[command(flatten)]
  add: CalcArgs,

  /// Mixture of Epoch timestamps in the given precision or date-time strings
  #[arg()]
  input: Vec<ConversionInput>,

  /// When supplying multiple timestamps what order to print them in
  #[arg(value_enum, long, short)]
  order: Option<Order>,
}

impl Handler for ConvArgs {
  fn handle<W, E>(&self, mut out: W, mut err: E) -> Result<(), io::Error>
  where
    W: Write,
    E: Write,
  {
    let into_tz = self.timezone.get();
    let maybe_datetimes = self
      .input
      .iter()
      // Extract as datetime
      .map(|inp| inp.to_dt(&self.format.precision))
      // Convert to the given timezone
      .map(|rdt| rdt.map(|dt| dt.with_timezone(&into_tz)))
      // Apply addition
      .map(|rdt| rdt.and_then(|dt| self.add.eval(dt)))
      .collect::<Result<Vec<_>, _>>();

    // Sus out any errors now that we're done oeprating
    let mut dts = match maybe_datetimes {
      Err(e) => return writeln!(&mut err, "{}", e),
      Ok(dts) => dts,
    };

    // Apply sorting rules
    dts.sort_by(|a, b| match self.order {
      Some(Order::Dsc) => Ord::cmp(&a, &b).reverse(),
      Some(Order::Asc) => Ord::cmp(&a, &b),
      None => Ordering::Equal,
    });

    // Apply output formatting
    dts
      .iter()
      .try_for_each(|dt| writeln!(&mut out, "{}", self.format.format(dt)))
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
      ConversionInput::String(dt) => Ok(*dt),
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
      run_test(" convert -t=America/New_York -p secs 1679258022 1676258186 1679258186 -o dsc -f");
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
    let (output, error) = run_test(" convert 1679258022 1676258187 1679258186");
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
    let (output, error) = run_test(" convert 1679258022 1676258187 1679258186 -o asc");
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
    let (output, error) = run_test(" convert 1679258022 1676258187 1679258186 -o dsc");
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
    let (output, error) = run_test(" convert 1679661279000 1679661179000 1679661079000");
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
    let (output, error) =
      run_test(" convert -p secs 1679258022 2023-03-19T16:36:26-0400 1679258186");
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
    let (output, error) = run_test(
      " convert 2023-03-19T16:36:26-0400 2023-03-19T16:33:42-0400 2023-02-12T22:16:26-0500",
    );
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
