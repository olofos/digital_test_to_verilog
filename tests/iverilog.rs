use assert_cmd::Command;

mod util;

fn iverilog_command(
    static_inputs: &[&str],
    generated_inputs: &[&std::path::Path],
    output: &std::path::Path,
) -> Command {
    let mut cmd = Command::new("iverilog");
    cmd.arg("-g2012");
    for input in static_inputs {
        cmd.arg(format!(
            "{}{input}",
            concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/")
        ));
    }

    for input in generated_inputs {
        cmd.arg(input);
    }
    cmd.arg("-o").arg(output);

    cmd
}

#[test_with::executable(iverilog)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn adder_simple_runs() {
        let dir = util::TempDir::create("adder_simple_runs");

        let file = dir.file("adder_simple_test.v");
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args([
            concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/adder.dig"),
            "0",
            "-o",
        ])
        .arg(&file)
        .assert()
        .success();

        assert!(file.exists());

        let exec_file = dir.file("out");

        let mut iverilog = iverilog_command(&["adder.v", "adder_scaffold.v"], &[&file], &exec_file);
        iverilog.assert().success();

        assert!(exec_file.exists());

        let mut cmd = Command::new(&exec_file);
        cmd.assert().success().stdout("All tests passed.\n");

        dir.delete();
    }

    #[test]
    fn adder_failure_fails_with_an_error() {
        let dir = util::TempDir::create("adder_failure_fails_with_an_error");

        let file = dir.file("adder_failure_fails_with_an_error.v");
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args([
            concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/adder.dig"),
            "1",
            "-o",
        ])
        .arg(&file)
        .assert()
        .success();

        assert!(file.exists());

        let exec_file = dir.file("out");

        let mut iverilog = iverilog_command(&["adder.v", "adder_scaffold.v"], &[&file], &exec_file);
        iverilog.assert().success();

        assert!(exec_file.exists());

        let mut cmd = Command::new(&exec_file);
        cmd.assert().failure();

        dir.delete();
    }

    #[test]
    fn test_74162_runs() {
        let dir = util::TempDir::create("test_74162_runs");

        let file = dir.file("74162.v");
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args([
            concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/74162.dig"),
            "0",
            "-o",
        ])
        .arg(&file)
        .assert()
        .success();

        assert!(file.exists());

        let exec_file = dir.file("out");

        let mut iverilog = iverilog_command(&["74162.v", "74162_scaffold.v"], &[&file], &exec_file);
        iverilog.assert().success();

        assert!(exec_file.exists());

        let mut cmd = Command::new(&exec_file);
        cmd.assert().success().stdout("All tests passed.\n");

        dir.delete();
    }

    #[rstest]
    #[case("A (A0000)")]
    #[case("A OR B (A0001)")]
    #[case("A OR !B (A0010)")]
    #[case("minus 1 (A0011)")]
    #[case("A plus (A AND !B) (A0100)")]
    #[case("(A OR B) plus (A AND !B) (A0101)")]
    #[case("A minus B minus 1 (A0110)")]
    #[case("(A AND !B) minus 1 (A0111)")]
    #[case("A plus (A AND B) (A1000)")]
    // Skip case #[case("A plus B (A1001)")]
    #[case("(A OR !B) plus (A AND B) (A1010)")]
    #[case("(A AND B) minus 1 (A1011)")]
    #[case("A plus A (A1100)")]
    #[case("(A OR B) plus A (A1101)")]
    #[case("(A OR !B) plus A (A1110)")]
    #[case("A minus 1 (A1111)")]
    #[case("!A (L0000)")]
    #[case("!A AND !B (L0001)")]
    #[case("!A AND B (L0010)")]
    #[case("logic 0 (L0011)")]
    #[case("!(A AND B) (L0100)")]
    #[case("!B (L0101)")]
    #[case("A XOR B (L0110)")]
    #[case("A AND !B (L0111)")]
    #[case("!A OR B (L1000)")]
    #[case("!(A XOR B) (L1001)")]
    #[case("B (L1010)")]
    #[case("A AND B (L1011)")]
    #[case("logic 1 (L1100)")]
    #[case("A OR !B (L1101)")]
    #[case("A OR B (L1110)")]
    #[case("A (L1111)")]

    fn test_74181_runs(#[case] name: &str) {
        let dir = util::TempDir::create(format!("test_74181_runs"));

        let file = dir.file("74181.v");
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args([
            concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/74181.dig"),
            name,
            "-o",
        ])
        .arg(&file)
        .assert()
        .success();

        assert!(file.exists());

        let exec_file = dir.file("out");

        let mut iverilog = iverilog_command(&["74181.v", "74181_scaffold.v"], &[&file], &exec_file);
        iverilog.assert().success();

        assert!(exec_file.exists());

        let mut cmd = Command::new(&exec_file);
        cmd.assert().success().stdout("All tests passed.\n");

        dir.delete();
    }
}
