use assert_cmd::Command;

mod util;

#[test]
fn cli_works() {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg("--help").assert().success();
}

fn expected_ouput(pre: &str, default: &str, delay1: &str, delay2: &str) -> String {
    format!(
        r#"{pre}`define assert_eq(signal, value) \
    if (signal !== value) begin \
        $display("ASSERTION FAILED in %m: signal != value"); \
    end

module tb (
    output reg [7:0] \A ,
    output reg [7:0] \B ,
    input [7:0] \S ,
    input  \C 
);
initial begin{default}
    \A = 1;
    \B = 1;
{delay1}
    `assert_eq(\S , 2);
{delay2}
end
endmodule
"#
    )
}

#[test]
fn can_load_test_file() {
    let expected_ouput = expected_ouput("", "", "#10;", "");

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.args([
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/adder.dig"),
        "1",
    ])
    .assert()
    .success()
    .stdout(expected_ouput);
}

#[test]
fn timescale_works() {
    let expected_ouput = expected_ouput("`timescale 1us/1ns\n\n", "", "#10;", "");

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.args([
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/adder.dig"),
        "1",
        "--timescale",
        "1us/1ns",
    ])
    .assert()
    .success()
    .stdout(expected_ouput);
}

#[test]
fn timescale_works_with_one_time() {
    let expected_ouput = expected_ouput("`timescale 1us/1us\n\n", "", "#10;", "");

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.args([
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/adder.dig"),
        "1",
        "--timescale",
        "1us",
    ])
    .assert()
    .success()
    .stdout(expected_ouput);
}

#[test]
fn timescale_gives_error_for_unknown_unit() {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.args([
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/adder.dig"),
        "1",
        "--timescale",
        "1km",
    ])
    .assert()
    .failure();
}

#[test]
fn delay_works() {
    let expected_ouput = expected_ouput("", "", "#20;", "#10;\n");

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.args([
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/adder.dig"),
        "1",
        "--delay",
        "20:10",
    ])
    .assert()
    .success()
    .stdout(expected_ouput);
}

#[test]
fn delay_works_with_one_number() {
    let expected_ouput = expected_ouput("", "", "#20;", "");

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.args([
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/adder.dig"),
        "1",
        "--delay",
        "20",
    ])
    .assert()
    .success()
    .stdout(expected_ouput);
}

#[test]
fn default_works() {
    let default = r#"
    \A = 0;
    \B = 0;
#10;
"#;
    let expected_ouput = expected_ouput("", default, "#10;", "");

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.args([
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/adder.dig"),
        "1",
        "--default",
    ])
    .assert()
    .success()
    .stdout(expected_ouput);
}

#[test]
fn output_to_file_works() {
    use std::io::Read;
    let dir = util::TempDir::create("output_to_file_works");

    let expected_ouput = expected_ouput("", "", "#10;", "");
    let path = dir.file("output_to_file_works.v");

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.args([
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/adder.dig"),
        "1",
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
    assert_eq!(content, expected_ouput);

    dir.delete();
}
