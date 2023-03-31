mod convert;
mod current;
mod hduration;
mod timezone;
mod types;

use clap::{Parser, Subcommand};
use convert::ConvArgs;
use current::CurrentArgs;
use std::io::{self, Write};
use timezone::TzArgs;
use types::Handler;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(args_conflicts_with_subcommands = true)]
struct Cli {
  #[command(subcommand)]
  commands: Option<Commands>,

  #[command(flatten)]
  current: CurrentArgs,
}

#[derive(Subcommand)]
enum Commands {
  /// (default) Get the current epoch time
  Current(CurrentArgs),
  /// Convert a list of epoch timestamps into date strings or vice versa
  Convert(ConvArgs),
  /// Get information on supported timezones
  Timezone(TzArgs),
  // TODO: Delta. Eg get diff of N time-likes and print human legible
}

fn main() -> Result<(), io::Error> {
  let cli = Cli::parse();
  let output = io::stdout();
  let error = io::stderr();
  run(cli, output, error)
}

fn run<W, E>(cli: Cli, output: W, error: E) -> Result<(), io::Error>
where
  W: Write,
  E: Write,
{
  match cli.commands {
    Some(Commands::Timezone(tza)) => tza.handle(output, error),
    Some(Commands::Convert(conv)) => conv.handle(output, error),
    Some(Commands::Current(curr)) => curr.handle(output, error),
    None => cli.current.handle(output, error),
  }
}
