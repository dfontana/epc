use chrono::{DateTime, Datelike, Duration, DurationRound, FixedOffset};
use clap::Args;

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
  pub fn apply(&self, dt: DateTime<FixedOffset>) -> Result<DateTime<FixedOffset>, String> {
    let Some(field) = self.truncate.as_ref() else {
      return Ok(dt);
    };

    let trunc_dur = match field {
      Precision::Weeks | Precision::Days | Precision::Hours => Duration::days(1),
      Precision::Mins => Duration::hours(1),
      Precision::Secs => Duration::minutes(1),
      Precision::Millis => Duration::seconds(1),
      Precision::Nanos => Duration::milliseconds(1),
    };
    let trunc = dt
      .duration_trunc(trunc_dur)
      .map_err(|e| format!("Could not truncate: {}", e))?;
    let trunc = match field {
      Precision::Weeks => trunc.with_month(1).and_then(|v| v.with_day(1)),
      Precision::Days => trunc.with_day(1),
      _ => Some(trunc),
    };
    trunc.ok_or_else(|| "Failed to truncate weeks/days".into())
  }
}

#[cfg(test)]
mod test {
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
  fn apply(#[case] in_nanos: i64, #[case] pre: Precision, #[case] exp_nanos: i64) {
    let args = TruncateArgs {
      truncate: Some(pre),
    };
    let nanos = Precision::Nanos;
    let truncated_0 = args.apply(nanos.parse(in_nanos).unwrap().into());
    let truncated = truncated_0.map(|p| p.timestamp_nanos());
    assert_eq!(truncated, Ok(exp_nanos))
  }
}
