use chrono::Utc;
use clap::Args;
use std::io::{self, Write};

use crate::{
  common::{AtTimezoneArgs, FormatArgs},
  Handler,
};

#[derive(Args)]
pub struct CurrentArgs {
  #[command(flatten)]
  timezone: AtTimezoneArgs,

  #[command(flatten)]
  format: FormatArgs,
}

impl Handler for CurrentArgs {
  fn handle<W, E>(&self, mut out: W, _err: E) -> Result<(), io::Error>
  where
    W: Write,
    E: Write,
  {
    writeln!(
      &mut out,
      "{}",
      self
        .format
        .format(&Utc::now().with_timezone(&self.timezone.get()))
    )
  }
}
