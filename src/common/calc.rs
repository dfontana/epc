use clap::Args;
use jiff::{Span, Zoned};

#[derive(Args)]
pub struct CalcArgs {
  /// Add a human friendly duration to all times (can be negative)
  #[arg(long, short = 'a', allow_hyphen_values = true)]
  add: Option<Span>,
}

impl CalcArgs {
  pub fn eval(&self, dt: Zoned) -> Result<Zoned, String> {
    self.add.as_ref().map_or(Ok(dt.clone()), |span| {
      dt.checked_add(*span).map_err(|e| e.to_string())
    })
  }
}

#[cfg(test)]
mod test {
  use std::str::FromStr;

  use jiff::{Span, ToSpan};
  use rstest::*;

  #[rstest]
  #[case("1s", 1.seconds())]
  #[case("1 s", 1.seconds())]
  #[case("0s", 0.seconds())]
  #[case("1ns", 1.nanoseconds())]
  #[case("1s 10ns", 1.seconds().nanoseconds(10))]
  #[case("-1ns", -1.nanoseconds())]
  #[case("-1s 1ns", -1.seconds().nanoseconds(1))]
  #[case("5m", 5.minutes())]
  #[case("5h", 5.hours())]
  #[case("5d", 5.days())]
  #[case("5w", 5.weeks())]
  #[case("3w 5d 2h 10m 7s 1ns", 3.weeks().days(5).hours(2).minutes(10).seconds(7).nanoseconds(1))]
  #[case("3w5d2h", 3.weeks().days(5).hours(2))]
  fn parses_friendly_spans(#[case] input: &str, #[case] expected: Span) {
    assert_eq!(
      Span::from_str(input).unwrap().fieldwise(),
      expected.fieldwise()
    );
  }

  #[rstest]
  #[case("1s -1ns")]
  #[case("s1")]
  #[case("s 1")]
  #[case("10ns 1s")]
  #[case(" 1s")]
  fn rejects_invalid_spans(#[case] input: &str) {
    assert!(Span::from_str(input).is_err());
  }
}
