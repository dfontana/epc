use std::{str::FromStr, time::Duration};

#[derive(Clone, Debug, PartialEq)]
pub struct HDuration {
  pub inner: Duration,
  pub negative: bool,
}

impl HDuration {
  pub fn new(sec: u64, nano: u32, negative: bool) -> Self {
    HDuration {
      inner: Duration::new(sec, nano),
      negative,
    }
  }
}

impl FromStr for HDuration {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut sec: u64 = 0;
    let mut nano: u32 = 0;
    let mut is_neg = false;

    let mut dbuf: u64 = 0;
    let mut in_char = false;
    let mut cbuf = String::new();

    let mut chars = s.chars().enumerate().peekable();
    loop {
      let next = chars.peek();
      let (idx, c) = next.unwrap_or(&(0, ' '));

      let should_flush = next.is_none() || (c.is_ascii_digit() && in_char);
      match (should_flush, c) {
        (_, ' ') | (true, _) => {
          in_char = false;
          let (ds, dn) = flush(dbuf, &cbuf)?;
          sec = sec
            .checked_add(ds)
            .ok_or::<String>("Too large of duration".into())?;
          nano = nano
            .checked_add(dn)
            .ok_or::<String>("Too large of duration".into())?;
          dbuf = c.to_digit(10).unwrap_or(0) as u64;
          cbuf = String::new();
          if chars.peek().is_none() {
            break;
          }
        }
        (_, '-') if *idx == 0 => is_neg = true,
        (_, c) if c.is_ascii_digit() => dbuf = dbuf * 10 + c.to_digit(10).unwrap() as u64,
        (_, 'm' | 's' | 'n' | 'd' | 'h' | 'w') => {
          cbuf.push(*c);
          in_char = true;
        }
        _ => return Err(format!("Invalid character {} at pos {}", c, idx)),
      }
      chars.next();
    }
    Ok(HDuration::new(sec, nano, is_neg))
  }
}

fn flush(dbuf: u64, cbuf: &str) -> Result<(u64, u32), String> {
  let mut sec: u64 = 0;
  let mut nano: u32 = 0;
  match cbuf {
    "w" => sec += dbuf * 7 * 24 * 60 * 60,
    "d" => sec += dbuf * 24 * 60 * 60,
    "h" => sec += dbuf * 60 * 60,
    "m" => sec += dbuf * 60,
    "s" => sec += dbuf,
    "ms" => {
      sec += dbuf / 1000;
      let up = (dbuf % 1000)
        .try_into()
        .ok()
        .and_then(|d| nano.checked_add(d));
      match up {
        Some(v) => nano += v,
        None => return Err("Too many milliseconds provided".into()),
      }
    }
    "ns" => {
      sec += dbuf / 1000000000;
      let up = (dbuf % 1000000000)
        .try_into()
        .ok()
        .and_then(|d| nano.checked_add(d));
      match up {
        Some(v) => nano += v,
        None => return Err("Too many nanoseconds provided".into()),
      }
    }
    _ => return Err("Missing units".into()),
  }
  Ok((sec, nano))
}

#[cfg(test)]
mod test {
  use std::str::FromStr;

  use rstest::*;

  use super::HDuration;

  #[test]
  fn from_strt() {
    let input = "3w5d2h";
    let expected = HDuration::new(2253600, 0, false);
    assert_eq!(HDuration::from_str(input), Ok(expected))
  }

  #[rstest]
  #[case("1s", HDuration::new(1, 0, false))]
  #[case("0s", HDuration::new(0, 0, false))]
  #[case("1ns", HDuration::new(0, 1, false))]
  #[case("1s 10ns", HDuration::new(1, 10, false))]
  #[case("10ns 1s", HDuration::new(1, 10, false))]
  #[case("-1ns", HDuration::new(0, 1, true))]
  #[case("-1s 1ns", HDuration::new(1, 1, true))]
  #[case("5m", HDuration::new(300, 0, false))]
  #[case("5h", HDuration::new(18000, 0, false))]
  #[case("5d", HDuration::new(432000, 0, false))]
  #[case("5w", HDuration::new(3024000, 0, false))]
  #[case("3w 5d 2h 10m 7s 1ns", HDuration::new(2254207, 1, false))]
  #[case("3w5d2h", HDuration::new(2253600, 0, false))]
  fn from_str(#[case] input: &str, #[case] expected: HDuration) {
    assert_eq!(HDuration::from_str(input), Ok(expected))
  }

  #[rstest]
  #[case("1s -1ns")] // Negative must be at front
  #[case("s1")] // Wrong order
  #[case("1 s")]
  #[case("s 1")]
  #[case(" 1s")]
  fn invalid_from_str(#[case] input: &str) {
    assert!(HDuration::from_str(input).is_err())
  }
}
