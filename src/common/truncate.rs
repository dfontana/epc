use clap::Args;
use jiff::{civil::Time, RoundMode, Unit, Zoned, ZonedRound};

use super::Precision;

#[derive(Args)]
pub struct TruncateArgs {
  /// Truncate time starting from the given position onwards. Truncation
  /// specifically means zeroing the epoch timestamp from that precision
  /// onwards. This makes certain assumptions, like there being 7 days
  /// in a week, and 52 weeks in a year. Not all of these
  /// properties are globally true.
  #[arg(value_enum, long, short = 'u')]
  truncate: Option<Precision>,
}

impl TruncateArgs {
  pub fn apply(&self, dt: Zoned) -> Result<Zoned, String> {
    let Some(field) = self.truncate else {
      return Ok(dt);
    };
    let truncated = match field {
      Precision::Weeks => dt.with().month(1).day(1).time(Time::MIN).build(),
      Precision::Days => dt.with().day(1).time(Time::MIN).build(),
      Precision::Hours => dt.round(ZonedRound::new().smallest(Unit::Day).mode(RoundMode::Trunc)),
      Precision::Mins => dt.round(
        ZonedRound::new()
          .smallest(Unit::Hour)
          .mode(RoundMode::Trunc),
      ),
      Precision::Secs => dt.round(
        ZonedRound::new()
          .smallest(Unit::Minute)
          .mode(RoundMode::Trunc),
      ),
      Precision::Millis => dt.round(
        ZonedRound::new()
          .smallest(Unit::Second)
          .mode(RoundMode::Trunc),
      ),
      Precision::Nanos => dt.round(
        ZonedRound::new()
          .smallest(Unit::Millisecond)
          .mode(RoundMode::Trunc),
      ),
    };
    truncated.map_err(|e| format!("Could not truncate: {}", e))
  }
}

#[cfg(test)]
mod test {
  use jiff::{tz::TimeZone, Timestamp};
  use rstest::*;

  use crate::common::{Precision, TruncateArgs};

  #[rstest]
  #[case(1681330711220000120, Precision::Nanos, 1681330711220000000)]
  #[case(1681330711220000120, Precision::Millis, 1681330711000000000)]
  #[case(1681330711220000120, Precision::Secs, 1681330680000000000)]
  #[case(1681330711220000120, Precision::Mins, 1681329600000000000)]
  #[case(1681330711220000120, Precision::Hours, 1681257600000000000)]
  #[case(1681330711220000120, Precision::Days, 1680307200000000000)]
  #[case(1681330711220000120, Precision::Weeks, 1672531200000000000)]
  fn apply(#[case] in_nanos: i64, #[case] pre: Precision, #[case] exp_nanos: i128) {
    let args = TruncateArgs {
      truncate: Some(pre),
    };
    let dt = Timestamp::from_nanosecond(in_nanos.into())
      .unwrap()
      .to_zoned(TimeZone::UTC);
    let truncated = args.apply(dt).map(|p| p.timestamp().as_nanosecond());
    assert_eq!(truncated, Ok(exp_nanos))
  }
}
