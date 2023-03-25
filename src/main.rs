mod convert;
mod current;
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

#[cfg(test)]
mod test {
  use crate::{run, Cli};
  use clap::Parser;
  use indoc::indoc;

  fn run_test(cli_str: &str) -> (String, String) {
    let mut output = Vec::new();
    let mut error = Vec::new();
    let cli = Cli::try_parse_from(cli_str.split(' ')).expect("Could not parse args");
    run(cli, &mut output, &mut error).expect("Failed to run");
    let output = String::from_utf8(output).expect("Not UTF-8");
    let error = String::from_utf8(error).expect("Not UTF-8");
    (output, error)
  }

  #[test]
  fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert()
  }

  #[test]
  fn verify_stamp() {
    let (output, error) =
      run_test(" -p secs -t America/New_York 1679258022 1676258186 1679258186 -o dsc -f");
    assert_eq!("", error);
    assert_eq!(
      indoc! {"
        2023-03-19T16:36:26-0400
        2023-03-19T16:33:42-0400
        2023-02-12T22:16:26-0500
      "},
      output
    );
  }

  #[test]
  fn no_sort() {
    let (output, error) = run_test(" 1679258022 1676258187 1679258186");
    assert_eq!("", error);
    assert_eq!(
      indoc! {"
        1679258022
        1676258187
        1679258186
      "},
      output
    );
  }

  #[test]
  fn sort_asc() {
    let (output, error) = run_test(" 1679258022 1676258187 1679258186 -o asc");
    assert_eq!("", error);
    assert_eq!(
      indoc! {"
        1676258187
        1679258022
        1679258186
      "},
      output
    );
  }

  #[test]
  fn sort_dsc() {
    let (output, error) = run_test(" 1679258022 1676258187 1679258186 -o dsc");
    assert_eq!("", error);
    assert_eq!(
      indoc! {"
        1679258186
        1679258022
        1676258187
       "},
      output
    );
  }

  #[test]
  fn millis() {
    let (output, error) = run_test(" 1679661279000 1679661179000 1679661079000");
    assert_eq!("", error);
    assert_eq!(
      indoc! {"
        1679661279000
        1679661179000
        1679661079000
      "},
      output
    );
  }

  #[test]
  fn mixed_input() {
    let (output, error) = run_test(" -p secs 1679258022 2023-03-19T16:36:26-0400 1679258186");
    assert_eq!("", error);
    assert_eq!(
      indoc! {"
        1679258022
        1679258186
        1679258186
      "},
      output
    );
  }

  #[test]
  fn string_only() {
    let (output, error) =
      run_test(" 2023-03-19T16:36:26-0400 2023-03-19T16:33:42-0400 2023-02-12T22:16:26-0500");
    assert_eq!("", error);
    assert_eq!(
      indoc! {"
        1679258186000
        1679258022000
        1676258186000
      "},
      output
    );
  }
}
