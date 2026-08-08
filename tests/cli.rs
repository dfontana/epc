use std::process::Command;

fn run(args: &[&str]) -> String {
  let output = Command::new(env!("CARGO_BIN_EXE_epc"))
    .args(args)
    .output()
    .expect("CLI entrypoint should run");

  assert!(
    output.status.success(),
    "CLI failed with stderr: {}",
    String::from_utf8_lossy(&output.stderr),
  );
  assert_eq!(b"", output.stderr.as_slice());

  String::from_utf8(output.stdout).expect("CLI stdout should be UTF-8")
}

#[test]
fn formats_utc_with_z_using_the_default_format() {
  assert_eq!(
    "1970-01-01T00:00:00Z\n",
    run(&[
      "convert",
      "--at-timezone=UTC",
      "--precision=secs",
      "-f",
      "0",
    ]),
  );
}

#[test]
fn formats_an_explicit_utc_epoch_with_percent_plus() {
  assert_eq!(
    "1970-01-01T00:00:00+00:00\n",
    run(&[
      "convert",
      "--at-timezone=UTC",
      "--precision",
      "secs",
      "--output-format=%+",
      "0",
    ]),
  );
}

#[test]
fn formats_fractional_seconds_with_percent_plus() {
  assert_eq!(
    "1970-01-01T00:00:00.123+00:00\n",
    run(&[
      "convert",
      "--at-timezone=UTC",
      "--precision=millis",
      "--output-format=%+",
      "123",
    ]),
  );
}

#[test]
fn truncates_automatic_fixed_offset_input_before_timezone_conversion() {
  assert_eq!(
    "2023-03-18T22:00:00+0000\n",
    run(&[
      "convert",
      "--at-timezone=UTC",
      "--precision=secs",
      "--truncate=hours",
      "--output-format=%Y-%m-%dT%H:%M:%S%z",
      "2023-03-19T00:30:00+0200",
    ]),
  );
}

#[test]
fn parses_an_explicit_input_format_as_utc() {
  assert_eq!(
    "1689379200000\n",
    run(&[
      "convert",
      "--at-timezone=UTC",
      "--input-format=%Y-%m-%d",
      "2023-07-15",
    ]),
  );
}
