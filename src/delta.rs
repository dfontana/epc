use std::str::FromStr;

use chrono::{Duration, Utc};
use clap::{builder::PossibleValue, Args, ValueEnum};

use crate::{
  common::{FormatArgs, OrderArgs, Precision},
  convert::ConversionInput,
  hduration::HDuration,
  Handler,
};

#[derive(ValueEnum, Clone)]
enum OutputStructure {
  /// A vertical table cleanly showing the two elements and their associated
  /// delta, in a two column format. This is good for humans and short(er) input.
  ListTable,
  /// A CSV of just the differences. This is good for machine interactions.
  ValueCsv,
  /// A three column CSV, emitting the first value, the compared value, and their
  /// difference. This works well for Machines too.
  KeyValueCsv,
}

#[derive(Clone)]
enum OutputFormat {
  Human,
  Precision(Precision),
}

impl FromStr for OutputFormat {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    <Precision as ValueEnum>::from_str(s, true)
      .map(OutputFormat::Precision)
      .or_else(|_| {
        if s == "human" {
          Ok(OutputFormat::Human)
        } else {
          Err(format!("Unknown variant: {}", s))
        }
      })
  }
}

impl OutputFormat {
  pub fn possible_values() -> Vec<PossibleValue> {
    let mut items = vec![PossibleValue::new("human")
      .help("A human interpretable value, in the format used by add operations")];
    Precision::value_variants()
      .iter()
      .filter_map(|v| v.to_possible_value())
      .for_each(|v| items.push(v));
    items
  }

  fn apply(&self, diff: Duration) -> String {
    match self {
      OutputFormat::Precision(p) => format!("{} {}", p.as_self_lossy(diff), p),
      OutputFormat::Human => format!("{}", HDuration::from(diff)),
    }
  }
}

#[derive(Args)]
pub struct DeltaArgs {
  /// How to stucture the output
  #[arg(value_enum, long, short='s', default_value_t=OutputStructure::ListTable)]
  output_structure: OutputStructure,

  /// How to print the deltas
  #[arg(long, short='d', value_parser=OutputFormat::possible_values(), default_value="human")]
  output_delta_format: String,

  #[command(flatten)]
  format: FormatArgs,

  // TODO: Validate this is at least 1 in length
  /// Mixture of Epoch timestamps in the given precision or date-time strings
  #[arg()]
  input: Vec<ConversionInput>,

  #[command(flatten)]
  order: OrderArgs,
}

impl Handler for DeltaArgs {
  fn handle<W, E>(&self, mut out: W, mut err: E) -> Result<(), std::io::Error>
  where
    W: std::io::Write,
    E: std::io::Write,
  {
    // Parse the output format, since Clap doesn't have a solution for
    // handling a nested enum type like this
    let delta_format = match OutputFormat::from_str(&self.output_delta_format) {
      Ok(v) => v,
      Err(e) => return writeln!(&mut err, "{}", e),
    };

    // Convert to datetimes
    let maybe_datetimes = self
      .input
      .iter()
      .map(|inp| inp.to_dt(&self.format.precision))
      .collect::<Result<Vec<_>, _>>();
    let mut dts = match maybe_datetimes {
      Err(e) => return writeln!(&mut err, "{}", e),
      Ok(dts) => dts,
    };

    // Affix the current time if there is only one DT.
    if dts.len() == 1 {
      dts.push(Utc::now().into())
    }

    // Sort them by requested order
    self.order.apply(&mut dts);

    // Compute differences
    let diffs = dts.windows(2).map(|i| {
      let diff = if i[1] > i[0] {
        i[1] - i[0]
      } else {
        i[0] - i[1]
      };
      (diff, i[0], i[1])
    });

    // Apply the formats
    let mut diffs = diffs.map(|(diff, ia, ib)| {
      (
        delta_format.apply(diff),
        self.format.format(&ia),
        self.format.format(&ib),
      )
    });

    // Apply the output structure
    match self.output_structure {
      OutputStructure::ListTable => todo!(),
      OutputStructure::ValueCsv => diffs.enumerate().try_for_each(|(idx, (d, _, _))| {
        if idx == 0 {
          write!(&mut out, "{}", d)
        } else {
          write!(&mut out, ",{}", d)
        }
      }),
      OutputStructure::KeyValueCsv => {
        diffs.try_for_each(|(d, ia, ib)| writeln!(&mut out, "{},{},{}", d, ia, ib))
      }
    }
  }
}
