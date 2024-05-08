use assert_cmd::Command;

#[test]
fn cli_works() {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg("--help").assert().success();
}

#[test]
fn can_load_test_file() {
    let expected_ouput = r#"`define assert_eq(signal, value) \
    if (signal !== value) begin \
        $display("ASSERTION FAILED in %m: signal != value"); \
    end

module tb (
    output reg [7:0] \A ,
    output reg [7:0] \B ,
    input [7:0] \S ,
    input  \C 
);
initial begin
    \A = 1;
    \B = 1;
#10;
    `assert_eq(\S , 2);

end
endmodule
"#;

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.args([
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/test.dig"),
        "1",
    ])
    .assert()
    .success()
    .stdout(expected_ouput);
}

#[test]
fn timescale_works() {
    let expected_ouput = r#"`timescale 1us/1ns

`define assert_eq(signal, value) \
    if (signal !== value) begin \
        $display("ASSERTION FAILED in %m: signal != value"); \
    end

module tb (
    output reg [7:0] \A ,
    output reg [7:0] \B ,
    input [7:0] \S ,
    input  \C 
);
initial begin
    \A = 1;
    \B = 1;
#10;
    `assert_eq(\S , 2);

end
endmodule
"#;

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.args([
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/test.dig"),
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
    let expected_ouput = r#"`timescale 1us/1us

`define assert_eq(signal, value) \
    if (signal !== value) begin \
        $display("ASSERTION FAILED in %m: signal != value"); \
    end

module tb (
    output reg [7:0] \A ,
    output reg [7:0] \B ,
    input [7:0] \S ,
    input  \C 
);
initial begin
    \A = 1;
    \B = 1;
#10;
    `assert_eq(\S , 2);

end
endmodule
"#;

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.args([
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/test.dig"),
        "1",
        "--timescale",
        "1us",
    ])
    .assert()
    .success()
    .stdout(expected_ouput);
}

#[test]
fn delay_works() {
    let expected_ouput = r#"`define assert_eq(signal, value) \
    if (signal !== value) begin \
        $display("ASSERTION FAILED in %m: signal != value"); \
    end

module tb (
    output reg [7:0] \A ,
    output reg [7:0] \B ,
    input [7:0] \S ,
    input  \C 
);
initial begin
    \A = 1;
    \B = 1;
#20;
    `assert_eq(\S , 2);
#10;

end
endmodule
"#;

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.args([
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/test.dig"),
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
    let expected_ouput = r#"`define assert_eq(signal, value) \
    if (signal !== value) begin \
        $display("ASSERTION FAILED in %m: signal != value"); \
    end

module tb (
    output reg [7:0] \A ,
    output reg [7:0] \B ,
    input [7:0] \S ,
    input  \C 
);
initial begin
    \A = 1;
    \B = 1;
#20;
    `assert_eq(\S , 2);

end
endmodule
"#;

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.args([
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/test.dig"),
        "1",
        "--delay",
        "20",
    ])
    .assert()
    .success()
    .stdout(expected_ouput);
}
