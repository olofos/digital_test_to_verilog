use assert_cmd::Command;

mod util;

#[test]
fn cli_works() {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg("--help").assert().success();
}

fn expected_output(pre: &str, delay1: &str, delay2: &str) -> String {
    format!(
        r#"{pre}`define assert_eq(line_num, signal, value) \
    if (signal !== value) begin \
        $display("ASSERTION FAILED on line line_num: signal != value"); \
        error_count += 1; \
    end

module tb (
    output reg [7:0] A,
    output reg [7:0] B,
    input [7:0] \|S| ,
    input C
);
integer error_count = 0;
initial begin
    A = 1;
    B = 1;
{delay1}
    `assert_eq(2, \|S| , 2);
{delay2}
  if(error_count > 0) begin
    $display("There were failed assertions");
    $finish_and_return(1);
  end
  $display("All tests passed.");
end
endmodule
"#
    )
}

#[test]
fn can_load_test_file() {
    let expected_output = expected_output("", "#10;", "");

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.args([
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/adder.dig"),
        "0",
    ])
    .assert()
    .success()
    .stdout(expected_output);
}

#[test]
fn can_load_test_by_name() {
    let expected_output = expected_output("", "#10;", "");

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.args([
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/adder.dig"),
        "Simple",
    ])
    .assert()
    .success()
    .stdout(expected_output);
}

#[test]
fn timescale_works() {
    let expected_output = expected_output("`timescale 1us/1ns\n\n", "#10;", "");

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.args([
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/adder.dig"),
        "0",
        "--timescale",
        "1us/1ns",
    ])
    .assert()
    .success()
    .stdout(expected_output);
}

#[test]
fn timescale_works_with_one_time() {
    let expected_output = expected_output("`timescale 1us/1us\n\n", "#10;", "");

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.args([
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/adder.dig"),
        "0",
        "--timescale",
        "1us",
    ])
    .assert()
    .success()
    .stdout(expected_output);
}

#[test]
fn timescale_gives_error_for_unknown_unit() {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.args([
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/adder.dig"),
        "0",
        "--timescale",
        "1km",
    ])
    .assert()
    .failure();
}

#[test]
fn delay_works() {
    let expected_output = expected_output("", "#20;", "#10;\n");

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.args([
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/adder.dig"),
        "0",
        "--delay",
        "20:10",
    ])
    .assert()
    .success()
    .stdout(expected_output);
}

#[test]
fn delay_works_with_one_number() {
    let expected_output = expected_output("", "#20;", "");

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.args([
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/adder.dig"),
        "0",
        "--delay",
        "20",
    ])
    .assert()
    .success()
    .stdout(expected_output);
}

#[test]
fn output_to_file_works() {
    use std::io::Read;
    let dir = util::TempDir::create("output_to_file_works");

    let expected_output = expected_output("", "#10;", "");
    let path = dir.file("output_to_file_works.v");

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.args([
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/adder.dig"),
        "0",
        "-o",
    ])
    .arg(&path)
    .assert()
    .success()
    .stdout("");

    let mut file = std::fs::File::open(&path).expect("Could not open output file.");
    let mut content = String::new();
    file.read_to_string(&mut content)
        .expect("Could not read output file.");
    assert_eq!(content, expected_output);

    dir.delete();
}
