use digital_test_runner::{dig, ExpectedEntry, ExpectedValue, InputEntry, InputValue, SignalType};
use miette::IntoDiagnostic;
use verilog::{VerilogIdentifier, VerilogValue};

use clap::Parser;
use std::path::PathBuf;

mod verilog;

macro_rules! outputln {
    ($($t:tt)*) => {{
        writeln!($($t)*).into_diagnostic()
    }};
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TestCaseSelector {
    Number(usize),
    Name(String),
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path to dig file
    file: PathBuf,
    /// Select test case by name or (zero based) index. Optional if there is only a single test.
    test: Option<TestCaseSelector>,
    /// Output file. By default the output is written to stdout.
    #[arg(long, short, value_name = "FILE")]
    output: Option<PathBuf>,
    /// Verilog timescale, eg, 10ns or 1us/1us
    #[arg(long, short, value_parser = parse_timescale)]
    timescale: Option<String>,
    /// An argument such as "10:5" means dealy 10 ticks after setting inputs and 5 ticks after reading outputs. The second value is optional and defaults to zero.
    #[arg(long, short, default_value = "10:0", value_parser = parse_delay)]
    delay: (u32, u32),
}

impl std::str::FromStr for TestCaseSelector {
    type Err = &'static str; // The actual type doesn't matter since we never error, but it must implement `Display`
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(s.parse::<usize>()
            .map(Self::Number)
            .unwrap_or_else(|_| Self::Name(s.to_string())))
    }
}

fn parse_timescale(s: &str) -> Result<String, String> {
    const HALF: &str = "[0-9]+[munpf]?s";
    const FULL: &str = "[0-9]+[munpf]?s/[0-9]+[munpf]?s";

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
    line: usize,
    out: &mut Box<dyn std::io::Write>,
    inputs: impl Iterator<Item = &'a InputEntry<'a>>,
    outputs: impl Iterator<Item = &'a ExpectedEntry<'a>>,
    delay: (u32, u32),
) -> miette::Result<()> {
    for input in inputs {
        let identifier = VerilogIdentifier::from_input(input.signal);
        let value = VerilogValue::from(input.value);
        outputln!(out, "    {identifier} = {value};")?;
    }
    outputln!(out, "#{};", delay.0)?;
    for output in outputs {
        let identifier = VerilogIdentifier::from(output.signal);
        let value = VerilogValue::from(output.value);
        outputln!(out, "    `assert_eq({line}, {identifier}, {value});")?;
    }
    if delay.1 > 0 {
        outputln!(out, "#{};", delay.1)?;
    }
    outputln!(out)?;
    Ok(())
}

fn main() -> miette::Result<()> {
    let cli = Cli::parse();

    let path = cli.file;
    eprintln!("Loading {path:?}");
    let input = std::fs::read_to_string(&path).unwrap();
    let dig_file = dig::File::parse(&input).unwrap();

    let test_num = match cli.test {
        Some(TestCaseSelector::Number(test_num)) => test_num,
        Some(TestCaseSelector::Name(name)) => {
            if let Some(test_num) = dig_file
                .test_cases
                .iter()
                .position(|test_case| test_case.name == name)
            {
                test_num
            } else {
                miette::bail!("No test case \"{name}\" found");
            }
        }
        None => {
            if dig_file.test_cases.len() == 1 {
                0
            } else {
                eprintln!("There are more than one test case in {path:?}");
                for (i, test_case) in dig_file.test_cases.iter().enumerate() {
                    if test_case.name.is_empty() {
                        eprintln!("{i}: [No name]");
                    } else {
                        eprintln!("{i}: {}", test_case.name);
                    }
                }
                miette::bail!("Please specify a test case");
            }
        }
    };
    eprintln!("Loading test case #{test_num}");
    let test_case = dig_file.load_test(test_num)?;

    // Construct iterator before even opening the output file.
    // This avoids overwriting the output file if the test is not static.
    let it = test_case.try_iter_static()?;

    let mut out: Box<dyn std::io::Write> = if let Some(path) = cli.output {
        let Ok(file) = std::fs::File::create(&path) else {
            miette::bail!("Could not open file {path:?} for output");
        };
        eprintln!("Writing output to {path:?}");
        Box::new(file)
    } else {
        Box::new(std::io::stdout())
    };

    if let Some(timescale) = cli.timescale {
        outputln!(out, "`timescale {timescale}\n")?;
    }

    outputln!(
        out,
        r#"`define assert_eq(line_num, signal, value) \
    if (signal !== value) begin \
        $display("ASSERTION FAILED on line line_num: signal != value"); \
        error_count += 1; \
    end"#
    )?;
    outputln!(out)?;

    let ports = test_case
        .signals
        .iter()
        .map(|sig| {
            let io_type = match sig.typ {
                SignalType::Input { .. } => "output reg",
                SignalType::Output => "input",
                SignalType::Bidirectional { .. } => "inout",
                SignalType::Virtual { .. } => unreachable!(),
            };
            let width = if sig.bits > 1 {
                format!("[{}:0] ", sig.bits - 1)
            } else {
                String::from("")
            };
            format!("    {io_type} {width}{}", VerilogIdentifier::from(sig))
        })
        .collect::<Vec<_>>()
        .join(",\n");
    outputln!(out, "module tb (\n{ports}\n);")?;
    outputln!(out, "integer error_count = 0;")?;

    for sig in &test_case.signals {
        if sig.is_bidirectional() {
            outputln!(
                out,
                "reg {} = {};",
                VerilogIdentifier::from_input(sig),
                VerilogValue::from(InputValue::Z)
            )?;
        }
    }

    for sig in &test_case.signals {
        if sig.is_bidirectional() {
            outputln!(
                out,
                "assign {} = {};",
                VerilogIdentifier::from(sig),
                VerilogIdentifier::from_input(sig)
            )?;
        }
    }
    outputln!(out, "initial begin")?;

    for row in it {
        let row = row?;
        print_row(
            row.line,
            &mut out,
            row.inputs.iter().filter(|inp| inp.changed),
            row.expected
                .iter()
                .filter(|exp| exp.value != ExpectedValue::X),
            cli.delay,
        )?;
    }

    outputln!(out, "  if(error_count > 0) begin")?;
    outputln!(out, "    $display(\"There were failed assertions\");")?;
    outputln!(out, "    $finish_and_return(1);")?;
    outputln!(out, "  end")?;
    outputln!(out, "  $display(\"All tests passed.\");")?;

    outputln!(out, "end")?;
    outputln!(out, "endmodule")?;

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
