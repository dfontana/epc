use chrono::{DateTime, FixedOffset};
use clap::Args;

use super::Precision;

#[derive(Args)]
pub struct TruncateArgs {
  /// Truncate time starting from the given position onwards. Truncation
  /// specifically means setting the temporal field to zero.
  #[arg(value_enum, long, short = 'u')]
  truncate: Option<Precision>,
}

impl TruncateArgs {
  pub fn apply(&self, dt: DateTime<FixedOffset>) -> Result<DateTime<FixedOffset>, String> {
    let Some(field) = self.truncate.as_ref() else {
      return Ok(dt);
    };

    let (stamp, precision) = match field {
      Precision::Nanos => (dt.timestamp_nanos(), Precision::Nanos),
      Precision::Millis => (dt.timestamp_millis(), Precision::Millis),
      _ => (dt.timestamp(), Precision::Secs),
    };

    let new_stamp = match field {
      Precision::Hours => (stamp / (24 * 60 * 60)) * 24 * 60 * 60,
      Precision::Mins => (stamp / (60 * 60)) * 60 * 60,
      Precision::Secs => (stamp / 60) * 60,
      Precision::Millis => (stamp / 1000) * 1000,
      Precision::Nanos => (stamp / 1000000) * 1000000,
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
      position: Some(pre),
    };
    let nanos = Precision::Nanos;
    let truncated_0 = args.apply(nanos.parse(in_nanos).unwrap().into());
    let truncated = truncated_0.map(|p| p.timestamp_nanos());
    assert_eq!(truncated, Ok(exp_nanos))
  }
}
