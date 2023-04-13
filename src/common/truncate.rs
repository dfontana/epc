use chrono::{DateTime, FixedOffset};
use clap::Args;

use super::Precision;

#[derive(Args)]
pub struct TruncateArgs {
  /// Truncate time starting from the given position onwards. Truncation
  /// specifically means zeroing the epoch timestamp from that precision
  /// onwards. This makes certain assumptions, like there being 24 hours
  /// in a day, 7 days in a week, and 52 weeks in a year. Not all of these
  /// properties are globally true.
  #[arg(value_enum, long, short = 'u')]
  truncate: Option<Precision>,
}

impl TruncateArgs {
  pub fn apply(&self, dt: DateTime<FixedOffset>) -> Result<DateTime<FixedOffset>, String> {
    let Some(field) = self.truncate.as_ref() else {
      return Ok(dt);
    };

    let (stamp, precision) = field.as_self(&dt);
    let new_stamp = match field {
      Precision::Millis => (stamp / 1000) * 1000,
      Precision::Nanos => (stamp / 1000000) * 1000000,
      _ => {
        let per_next = field
          .try_upcast()
          .map(|p| p.seconds_per())
          .unwrap_or_else(|| Precision::highest_next() * Precision::highest().seconds_per());
        (stamp / per_next) * per_next
      }
    };

    precision
      .parse(new_stamp)
      .single()
      .map(|dt| dt.into())
      .ok_or_else(|| "Could not truncate".into())
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
