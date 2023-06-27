use std::{
  io::{self, Write},
  str::FromStr,
};

use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, TimeZone, Utc};
use clap::Args;

use crate::{
  common::{AtTimezoneArgs, CalcArgs, FormatArgs, OrderArgs, Precision, TruncateArgs},
  Handler,
};

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

  #[command(flatten)]
  order: OrderArgs,
}

impl Handler for ConvArgs {
  fn handle<W, E>(&self, mut out: W, mut err: E) -> Result<(), io::Error>
  where
    W: Write,
    E: Write,
  {
    let into_tz = self.timezone.get();
    let input_format = self.input_format.as_deref();

    let maybe_datetimes = self
      .input
      .iter()
      // Parse with custom format or auto-detect
      .map(|inp| ConversionInput::from_str_with_format(inp, input_format))
      // Extract as datetime
      .map(|rdt| rdt.and_then(|inp| inp.to_dt(&self.format.precision)))
      .map(|rdt| rdt.and_then(|dt| self.truncate.apply(dt)))
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
    self.order.apply(&mut dts);

    // Apply output formatting
    dts
      .iter()
      .try_for_each(|dt| writeln!(&mut out, "{}", self.format.format(dt)))
  }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConversionInput {
  Stamp(i64),
  String(DateTime<FixedOffset>),
}

impl ConversionInput {
  /// Parse with optional custom format
  fn from_str_with_format(arg: &str, format: Option<&str>) -> Result<Self, String> {
    // Try timestamp first (always)
    if let Ok(ts) = arg.parse::<i64>() {
      return Ok(ConversionInput::Stamp(ts));
    }

    match format {
      Some(fmt) => {
        // Try parsing with timezone first
        if let Ok(dt) = DateTime::parse_from_str(arg, fmt) {
          return Ok(ConversionInput::String(dt));
        }

        // Fall back to naive datetime (assume UTC)
        if let Ok(naive) = NaiveDateTime::parse_from_str(arg, fmt) {
          let dt_utc: DateTime<FixedOffset> = Utc.from_utc_datetime(&naive).into();
          return Ok(ConversionInput::String(dt_utc));
        }

        // Try date-only parsing (assume midnight UTC)
        if let Ok(date) = NaiveDate::parse_from_str(arg, fmt) {
          let naive = date.and_hms_opt(0, 0, 0).ok_or("Invalid date")?;
          let dt_utc: DateTime<FixedOffset> = Utc.from_utc_datetime(&naive).into();
          return Ok(ConversionInput::String(dt_utc));
        }

        Err(format!("Could not parse '{}' with format '{}'", arg, fmt))
      }
      None => {
        // Existing auto-detection logic
        match arg.parse::<DateTime<FixedOffset>>() {
          Ok(dt) => Ok(ConversionInput::String(dt)),
          Err(_) => Err(format!("Could not parse: {}", arg)),
        }
      }
    }
  }

  pub fn to_dt(&self, precision: &Precision) -> Result<DateTime<FixedOffset>, String> {
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
  use super::ConversionInput;
  use crate::{run, Cli};
  use chrono::DateTime;
  use clap::Parser;
  use indoc::indoc;
  use rstest::*;

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
  #[case("2023-07-15", "%Y-%m-%d", Some("2023-07-15T00:00:00+00:00"))]
  #[case("2023/07/15", "%Y/%m/%d", Some("2023-07-15T00:00:00+00:00"))]
  #[case("15.07.2023", "%d.%m.%Y", Some("2023-07-15T00:00:00+00:00"))]
  #[case(
    "2023-07-15 14:30:45",
    "%Y-%m-%d %H:%M:%S",
    Some("2023-07-15T14:30:45+00:00")
  )]
  #[case(
    "2023-07-15T14:30:45",
    "%Y-%m-%dT%H:%M:%S",
    Some("2023-07-15T14:30:45+00:00")
  )]
  #[case(
    "2023-07-15 14:30:45.123",
    "%Y-%m-%d %H:%M:%S%.3f",
    Some("2023-07-15T14:30:45.123+00:00")
  )]
  #[case(
    "2023-07-15 14:30:45+02:00",
    "%Y-%m-%d %H:%M:%S%:z",
    Some("2023-07-15T14:30:45+02:00")
  )]
  #[case(
    "2023-07-15 14:30:45 +0200",
    "%Y-%m-%d %H:%M:%S %z",
    Some("2023-07-15T14:30:45+02:00")
  )]
  #[case("20230715_143045", "%Y%m%d_%H%M%S", Some("2023-07-15T14:30:45+00:00"))]
  #[case(
    "07.15.2023 14:30",
    "%m.%d.%Y %H:%M",
    Some("2023-07-15T14:30:00+00:00")
  )]
  #[case("2023-07-15", "%Y-%m", None)] // Format mismatch
  #[case("invalid", "%Y-%m-%d", None)] // Invalid input
  #[case("2023-13-15", "%Y-%m-%d", None)] // Invalid month
  fn test_custom_format_parsing(
    #[case] input: &str,
    #[case] format: &str,
    #[case] expected_str: Option<&str>,
  ) {
    let result = ConversionInput::from_str_with_format(input, Some(format));

    match expected_str {
      Some(estr) => {
        let expected = ConversionInput::String(
          DateTime::parse_from_rfc3339(estr).expect("Test error, invalid expected"),
        );
        assert!(
          result.is_ok(),
          "Failed to parse '{}' with format '{}': {:?}",
          input,
          format,
          result
        );
        let conversion = result.unwrap();
        assert_eq!(
          conversion, expected,
          "Parsed datetime doesn't match expected for input '{}' with format '{}'",
          input, format
        );
      }
      None => {
        assert!(
          result.is_err(),
          "Expected parsing to fail for input '{}' with format '{}', but got: {:?}",
          input,
          format,
          result
        );
      }
    }
  }

  #[rstest]
  #[case("2023-07-15 14:30:45", "%Y-%m-%d %H:%M:%S")] // Naive -> UTC
  #[case("2023-07-15 14:30:45+02:00", "%Y-%m-%d %H:%M:%S%:z")] // With offset
  fn test_timezone_handling(#[case] input: &str, #[case] format: &str) {
    let result = ConversionInput::from_str_with_format(input, Some(format));
    assert!(result.is_ok(), "Failed to parse: {}", input);

    if let Ok(ConversionInput::String(dt)) = result {
      // Should have valid timezone information
      let offset_secs = dt.offset().local_minus_utc().abs();
      assert!(
        offset_secs <= 24 * 3600,
        "Invalid offset: {} seconds",
        offset_secs
      );
    }
  }

  #[test]
  fn test_cli_with_input_format_basic() {
    let (output, error) = run_test(" convert -i %Y-%m-%d 2023-07-15 2023-07-16");
    assert_eq!("", error);
    // Should output timestamps for the parsed dates
    assert!(output.contains("1689379200000")); // 2023-07-15 as millis
    assert!(output.contains("1689465600000")); // 2023-07-16 as millis
  }

  #[test]
  fn test_cli_with_input_format_time() {
    // Test datetime format without quotes to avoid shell parsing issues in test
    let (output, error) = run_test(" convert -i %Y-%m-%dT%H:%M:%S 2023-07-15T14:30:45");
    assert_eq!("", error);
    assert!(output.contains("1689431445000")); // Expected timestamp
  }

  #[test]
  fn test_cli_mixed_input_with_format() {
    let (output, error) = run_test(" convert -i %Y-%m-%d 1679258022 2023-07-15");
    assert_eq!("", error);
    // Should handle both timestamp and formatted date
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
}
