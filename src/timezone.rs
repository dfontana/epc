use std::io::{self, Write};

use clap::Args;
use jiff::tz;

use crate::Handler;

#[derive(Args)]
pub struct TzArgs {}

impl Handler for TzArgs {
  fn handle<W, E>(&self, mut out: W, _err: E) -> Result<(), io::Error>
  where
    W: Write,
    E: Write,
  {
    tz::db()
      .available()
      .try_for_each(|name| writeln!(&mut out, "{}", name))
  }
}
