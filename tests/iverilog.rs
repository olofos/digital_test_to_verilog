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
    #[case(0)]
    #[case(1)]
    #[case(2)]
    #[case(3)]
    #[case(4)]
    #[case(5)]
    #[case(6)]
    #[case(7)]
    #[case(8)]
    // Skip case 9 because it involves one extra signal, thus requiring a different scaffold for now
    #[case(10)]
    #[case(11)]
    #[case(12)]
    #[case(13)]
    #[case(14)]
    #[case(15)]
    #[case(16)]
    #[case(17)]
    #[case(18)]
    #[case(19)]
    #[case(20)]
    #[case(21)]
    #[case(22)]
    #[case(23)]
    #[case(24)]
    #[case(25)]
    #[case(26)]
    #[case(27)]
    #[case(28)]
    #[case(29)]
    #[case(30)]
    #[case(31)]

    fn test_74181_runs(#[case] test_num: usize) {
        let dir = util::TempDir::create(format!("test_74181_{test_num}_runs"));

        let file = dir.file("74181.v");
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args([
            concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/74181.dig"),
            format!("{test_num}").as_str(),
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
