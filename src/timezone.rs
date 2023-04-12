use std::io::{self, Write};

use chrono_tz::TZ_VARIANTS;
use clap::Args;

use crate::Handler;

#[derive(Args)]
pub struct TzArgs {}

impl Handler for TzArgs {
  fn handle<W, E>(&self, mut out: W, _err: E) -> Result<(), io::Error>
  where
    W: Write,
    E: Write,
  {
    TZ_VARIANTS
      .iter()
      .try_for_each(|f| writeln!(&mut out, "{}", f))
  }
}
