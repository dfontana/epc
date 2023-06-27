use std::cmp::Ordering;

use clap::{Args, ValueEnum};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Order {
  /// Ascending in time
  Asc,
  /// Descending in time
  Dsc,
}

#[derive(Args)]
pub struct OrderArgs {
  /// When supplying multiple timestamps what order to print them in
  #[arg(value_enum, long, short)]
  order: Option<Order>,
}

impl OrderArgs {
  pub fn apply<T>(&self, items: &mut [T])
  where
    T: Ord,
  {
    items.sort_by(|a, b| match self.order {
      Some(Order::Dsc) => Ord::cmp(&a, &b).reverse(),
      Some(Order::Asc) => Ord::cmp(&a, &b),
      None => Ordering::Equal,
    });
  }
}
