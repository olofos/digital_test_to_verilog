use digital_test_runner::{dig, DataEntry, InputValue, OutputValue, SignalDirection, TestCase};

use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    file: PathBuf,
    #[arg(default_value_t = 0)]
    test_num: usize,
    #[arg(long, short, value_parser = parse_timescale)]
    timescale: Option<String>,
    #[arg(long)]
    default: bool,
    #[arg(long, short, default_value = "10:0", value_parser = parse_delay)]
    delay: (u32, u32),
}

fn parse_timescale(s: &str) -> Result<String, String> {
    const HALF: &str = "[0-9]+[munp]?s";
    const FULL: &str = "[0-9]+[munp]?s/[0-9]+[munp]?s";

    let r = regex::Regex::new(FULL).unwrap();
    if r.is_match(s) {
        return Ok(s.to_string());
    }

    let r = regex::Regex::new(HALF).unwrap();
    if r.is_match(s) {
        return Ok(format!("{s}/{s}"));
    }

    Err(String::from("unknown timestamp format"))
}

fn parse_delay(s: &str) -> Result<(u32, u32), String> {
    let mut it = s.split(':');
    let Some(s1) = it.next() else {
        return Err(String::from("unexpected empty string"));
    };
    let Ok(d1) = s1.parse() else {
        return Err(format!("expected an integer, found {s1}"));
    };
    let d2 = if let Some(s2) = it.next() {
        let Ok(d2) = s2.parse() else {
            return Err(format!("expected an integer, found {s1}"));
        };
        d2
    } else {
        0
    };
    Ok((d1, d2))
}

fn print_row<'a>(
    inputs: impl Iterator<Item = DataEntry<'a, InputValue>>,
    outputs: impl Iterator<Item = DataEntry<'a, OutputValue>>,
    delay: (u32, u32),
) {
    for input in inputs {
        println!("    \\{} = {};", input.name, input.value);
    }
    println!("#{};", delay.0);
    for output in outputs {
        println!("    `assert_eq(\\{} , {});", output.name, output.value);
    }
    if delay.1 > 0 {
        println!("#{};", delay.1);
    }
    println!();
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let path = cli.file;
    let test_num = cli.test_num;
    eprintln!("Loading {path:?} test #{test_num}");
    let input = std::fs::read_to_string(path).unwrap();
    let dig_file = dig::parse(&input).unwrap();

    let test_case = TestCase::try_from_static_dig(&dig_file, test_num)?;

    if let Some(timescale) = cli.timescale {
        println!("`timescale {timescale}\n");
    }

    println!(
        r#"`define assert_eq(signal, value) \
    if (signal !== value) begin \
        $display("ASSERTION FAILED in %m: signal != value"); \
    end"#
    );
    println!();

    let ports = test_case
        .signals
        .iter()
        .map(|sig| {
            let io_type = match sig.dir {
                SignalDirection::Input { .. } => "output reg",
                SignalDirection::Output => "input",
            };
            let width = if sig.bits > 1 {
                format!("[{}:0]", sig.bits - 1)
            } else {
                String::from("")
            };
            format!("    {io_type} {width} \\{} ", sig.name)
        })
        .collect::<Vec<_>>()
        .join(",\n");
    println!("module tb (\n{ports}\n);");
    println!("initial begin");

    let mut iter = test_case.iter().skip(if cli.default { 0 } else { 1 });

    if let Some(row) = iter.next() {
        print_row(row.inputs(), row.checked_outputs(), cli.delay);
    }

    for row in iter {
        print_row(row.changed_inputs(), row.checked_outputs(), cli.delay);
    }

    println!("end");
    println!("endmodule");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert()
    }
}
