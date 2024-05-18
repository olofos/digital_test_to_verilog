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

    #[test]
    fn adder_simple_runs() {
        let dir = util::TempDir::create("adder_simple_runs");

        let file = dir.file("adder_simple_test.v");
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
        cmd.assert().success().stdout("All tests passed.\n");
    }
}
