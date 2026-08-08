use clap::Args;
use jiff::{tz::TimeZone, Timestamp};
use std::io::{self, Write};

use crate::{
  common::{AtTimezoneArgs, CalcArgs, FormatArgs, TruncateArgs},
  Handler,
};

#[derive(Args)]
pub struct CurrentArgs {
  #[command(flatten)]
  timezone: AtTimezoneArgs,

  #[command(flatten)]
  format: FormatArgs,

  #[command(flatten)]
  add: CalcArgs,

  #[command(flatten)]
  truncate: TruncateArgs,
}

impl Handler for CurrentArgs {
  fn handle<W, E>(&self, mut out: W, mut err: E) -> Result<(), io::Error>
  where
    W: Write,
    E: Write,
  {
    let rdt = self
      .truncate
      .apply(Timestamp::now().to_zoned(TimeZone::UTC))
      .map(|dt| dt.with_time_zone(self.timezone.get()))
      .and_then(|dt| self.add.eval(dt));
    let dt = match rdt {
      Err(e) => return write!(&mut err, "{}", e),
      Ok(v) => v,
    };
    writeln!(&mut out, "{}", self.format.format(&dt))
  }
}
