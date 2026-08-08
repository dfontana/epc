use std::{
  cmp::Ordering,
  io::{self, Write},
};

use clap::{Args, ValueEnum};
use jiff::{fmt::strtime, tz::TimeZone, Timestamp, Zoned};

use crate::{
  common::{AtTimezoneArgs, CalcArgs, FormatArgs, Precision, TruncateArgs},
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

  #[command(flatten)]
  truncate: TruncateArgs,

  /// Mixture of Epoch timestamps in the given precision or date-time strings
  #[arg()]
  input: Vec<String>,

  /// Input format for datetime strings (strftime-style).
  /// When specified, all string inputs must match this format.
  /// If not specified, auto-detects timestamps and RFC3339/ISO8601 formats.
  /// When a timezone is omitted, it's assumed UTC
  /// When a time is omitted, it's assumed Midnight UTC
  ///
  /// %Y(year) %m(month) %d(day) %H(hour) %M(min) %S(sec) %.3f(millis) %z(tz) %:z(tz+colon)
  ///
  /// Examples: "%Y-%m-%d" "%d/%m/%Y %H:%M:%S" "%Y-%m-%dT%H:%M:%S%:z"
  #[arg(long, short = 'i')]
  input_format: Option<String>,

  /// When supplying multiple timestamps what order to print them in
  #[arg(value_enum, long, short)]
  order: Option<Order>,

  /// Display results in a table format with input and output columns
  #[arg(long)]
  table: bool,
}

impl Handler for ConvArgs {
  fn handle<W, E>(&self, mut out: W, mut err: E) -> Result<(), io::Error>
  where
    W: Write,
    E: Write,
  {
    let into_tz = self.timezone.get();
    let input_format = self.input_format.as_deref();

    let maybe_pairs = self
      .input
      .iter()
      .map(|inp| {
        ConversionInput::from_str_with_format(inp, input_format)
          .and_then(|ci| ci.to_dt(&self.format.precision))
          .and_then(|dt| self.truncate.apply(dt))
          .map(|dt| dt.with_time_zone(into_tz.clone()))
          .and_then(|dt| self.add.eval(dt))
          .map(|dt| (inp, dt))
      })
      .collect::<Result<Vec<_>, _>>();

    let mut pairs = match maybe_pairs {
      Err(e) => return writeln!(&mut err, "{}", e),
      Ok(pairs) => pairs,
    };

    pairs.sort_by(|(_, a), (_, b)| match self.order {
      Some(Order::Dsc) => Ord::cmp(a, b).reverse(),
      Some(Order::Asc) => Ord::cmp(a, b),
      None => Ordering::Equal,
    });

    if self.table {
      self.format_table(&mut out, &pairs)
    } else {
      pairs
        .iter()
        .try_for_each(|(_, dt)| writeln!(&mut out, "{}", self.format.format(dt)))
    }
  }
}

impl ConvArgs {
  fn format_table<W>(&self, out: &mut W, pairs: &[(&String, Zoned)]) -> Result<(), io::Error>
  where
    W: Write,
  {
    if pairs.is_empty() {
      return Ok(());
    }

    let outputs: Vec<String> = pairs.iter().map(|(_, dt)| self.format.format(dt)).collect();
    let input_width = pairs
      .iter()
      .map(|(s, _)| s.len())
      .max()
      .unwrap_or(0)
      .max("Input".len());
    let output_width = outputs
      .iter()
      .map(|s| s.len())
      .max()
      .unwrap_or(0)
      .max("Output".len());

    writeln!(out, "┌─{:─<input_width$}─┬─{:─<output_width$}─┐", "", "")?;
    writeln!(
      out,
      "│ {:input_width$} │ {:output_width$} │",
      "Input", "Output"
    )?;
    writeln!(out, "├─{:─<input_width$}─┼─{:─<output_width$}─┤", "", "")?;

    for ((input, _), output) in pairs.iter().zip(outputs.iter()) {
      writeln!(out, "│ {:input_width$} │ {:output_width$} │", input, output)?;
    }

    writeln!(out, "└─{:─<input_width$}─┴─{:─<output_width$}─┘", "", "")?;
    Ok(())
  }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ConversionInput {
  Stamp(i64),
  String(Zoned),
}

impl ConversionInput {
  fn from_str_with_format(arg: &str, format: Option<&str>) -> Result<Self, String> {
    if let Ok(ts) = arg.parse::<i64>() {
      return Ok(ConversionInput::Stamp(ts));
    }

    match format {
      Some(fmt) => {
        let parsed = strtime::parse(fmt, arg)
          .map_err(|_| format!("Could not parse '{}' with format '{}'", arg, fmt))?;
        let zoned = parsed
          .to_zoned()
          .or_else(|_| {
            parsed
              .to_datetime()
              .and_then(|dt| dt.to_zoned(TimeZone::UTC))
          })
          .map_err(|_| format!("Could not parse '{}' with format '{}'", arg, fmt))?;
        Ok(ConversionInput::String(zoned))
      }
      None => Zoned::strptime("%Y-%m-%dT%H:%M:%S%.f%z", arg)
        .or_else(|_| Zoned::strptime("%Y-%m-%dT%H:%M:%S%.f%:z", arg))
        .or_else(|_| {
          arg
            .parse::<Timestamp>()
            .map(|timestamp| timestamp.to_zoned(TimeZone::UTC))
        })
        .map(ConversionInput::String)
        .map_err(|_| format!("Could not parse: {}", arg)),
    }
  }

  fn to_dt(&self, precision: &Precision) -> Result<Zoned, String> {
    match self {
      ConversionInput::String(dt) => Ok(dt.clone()),
      ConversionInput::Stamp(ts) => precision
        .parse(*ts)
        .map(|timestamp| timestamp.to_zoned(TimeZone::UTC))
        .map_err(|_| format!("Could not parse: {}", ts)),
    }
  }
}

#[cfg(test)]
mod test {
  use clap::Parser;
  use indoc::indoc;
  use jiff::Timestamp;
  use rstest::*;

  use super::ConversionInput;
  use crate::{run, Cli};

  fn run_test(cli_str: &str) -> (String, String) {
    let mut output = Vec::new();
    let mut error = Vec::new();
    let cli = Cli::try_parse_from(cli_str.split(' ')).expect("Could not parse args");
    run(cli, &mut output, &mut error).expect("Failed to run");
    let output = String::from_utf8(output).expect("Not UTF-8");
    let error = String::from_utf8(error).expect("Not UTF-8");
    (output, error)
  }

  #[rstest]
  #[case("2023-07-15", "%Y-%m-%d", Some("2023-07-15T00:00:00Z"))]
  #[case("2023/07/15", "%Y/%m/%d", Some("2023-07-15T00:00:00Z"))]
  #[case("15.07.2023", "%d.%m.%Y", Some("2023-07-15T00:00:00Z"))]
  #[case(
    "2023-07-15 14:30:45",
    "%Y-%m-%d %H:%M:%S",
    Some("2023-07-15T14:30:45Z")
  )]
  #[case(
    "2023-07-15T14:30:45",
    "%Y-%m-%dT%H:%M:%S",
    Some("2023-07-15T14:30:45Z")
  )]
  #[case(
    "2023-07-15 14:30:45.123",
    "%Y-%m-%d %H:%M:%S%.3f",
    Some("2023-07-15T14:30:45.123Z")
  )]
  #[case(
    "2023-07-15 14:30:45+02:00",
    "%Y-%m-%d %H:%M:%S%:z",
    Some("2023-07-15T12:30:45Z")
  )]
  #[case(
    "2023-07-15 14:30:45 +0200",
    "%Y-%m-%d %H:%M:%S %z",
    Some("2023-07-15T12:30:45Z")
  )]
  #[case("20230715_143045", "%Y%m%d_%H%M%S", Some("2023-07-15T14:30:45Z"))]
  #[case("07.15.2023 14:30", "%m.%d.%Y %H:%M", Some("2023-07-15T14:30:00Z"))]
  #[case("2023-07-15", "%Y-%m", None)]
  #[case("invalid", "%Y-%m-%d", None)]
  #[case("2023-13-15", "%Y-%m-%d", None)]
  fn test_custom_format_parsing(
    #[case] input: &str,
    #[case] format: &str,
    #[case] expected: Option<&str>,
  ) {
    let result = ConversionInput::from_str_with_format(input, Some(format));
    match expected {
      Some(expected) => {
        let ConversionInput::String(dt) = result.expect("input should parse") else {
          panic!("formatted input must be a date-time");
        };
        assert_eq!(dt.timestamp(), expected.parse::<Timestamp>().unwrap());
      }
      None => assert!(result.is_err(), "input should not parse"),
    }
  }

  #[rstest]
  #[case("2023-07-15 14:30:45", "%Y-%m-%d %H:%M:%S")]
  #[case("2023-07-15 14:30:45+02:00", "%Y-%m-%d %H:%M:%S%:z")]
  fn test_timezone_handling(#[case] input: &str, #[case] format: &str) {
    let ConversionInput::String(dt) =
      ConversionInput::from_str_with_format(input, Some(format)).expect("input should parse")
    else {
      panic!("formatted input must be a date-time");
    };
    assert!(dt.offset().seconds().abs() <= 24 * 3600);
  }

  #[test]
  fn test_cli_with_input_format_basic() {
    let (output, error) = run_test(" convert -i %Y-%m-%d 2023-07-15 2023-07-16");
    assert_eq!("", error);
    assert!(output.contains("1689379200000"));
    assert!(output.contains("1689465600000"));
  }

  #[test]
  fn test_cli_with_input_format_time() {
    let (output, error) = run_test(" convert -i %Y-%m-%dT%H:%M:%S 2023-07-15T14:30:45");
    assert_eq!("", error);
    assert!(output.contains("1689431445000"));
  }

  #[test]
  fn test_cli_mixed_input_with_format() {
    let (output, error) = run_test(" convert -i %Y-%m-%d 1679258022 2023-07-15");
    assert_eq!("", error);
    assert!(output.contains("1679258022"));
    assert!(output.contains("1689379200000"));
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

  #[test]
  fn table_format_basic() {
    let (output, error) = run_test(" convert --table 1679258022 1676258186 1679258186");
    assert_eq!("", error);
    assert!(output.contains("Input"));
    assert!(output.contains("Output"));
    assert!(output.contains("1679258022"));
    assert!(output.contains("1676258186"));
    assert!(output.contains("┌─"));
    assert!(output.contains("└─"));
  }

  #[test]
  fn table_format_with_formatting() {
    let (output, error) = run_test(" convert --table -f -p secs 1679258022 1676258186");
    assert_eq!("", error);
    assert!(output.contains("Input"));
    assert!(output.contains("Output"));
    assert!(output.contains("1679258022"));
    assert!(output.contains("1676258186"));
    assert!(output.contains("2023-03-19T20:33:42"));
    assert!(output.contains("2023-02-13T03:16:26"));
  }

  #[test]
  fn table_format_with_sorting() {
    let (output, error) =
      run_test(" convert --table -f -p secs -o dsc 1679258022 1676258186 1679258186");
    assert_eq!("", error);
    let data_rows: Vec<&str> = output
      .lines()
      .filter(|line| line.contains("2023-"))
      .collect();
    assert_eq!(3, data_rows.len());
    assert!(data_rows[0].contains("2023-03-19T20:36:26"));
  }

  #[test]
  fn table_format_with_custom_input() {
    let (output, error) = run_test(" convert --table -i %Y-%m-%d -f 2023-07-15 2023-07-16");
    assert_eq!("", error);
    assert!(output.contains("2023-07-15"));
    assert!(output.contains("2023-07-16"));
    assert!(output.contains("2023-07-15T00:00:00"));
    assert!(output.contains("2023-07-16T00:00:00"));
  }
}
