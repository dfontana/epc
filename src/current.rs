use chrono::Utc;
use clap::Args;
use std::io::{self, Write};

use crate::{
  common::{AtTimezoneArgs, CalcArgs, FormatArgs},
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
}

impl Handler for CurrentArgs {
  fn handle<W, E>(&self, mut out: W, mut err: E) -> Result<(), io::Error>
  where
    W: Write,
    E: Write,
  {
    let rdt = self
      .add
      .eval(Utc::now().with_timezone(&self.timezone.get()));
    let dt = match rdt {
      Err(e) => return write!(&mut err, "{}", e),
      Ok(v) => v,
    };
    writeln!(&mut out, "{}", self.format.format(&dt))
  }
}
