use digital_test_runner::{dig, DataEntry, InputValue, OutputValue, SignalDirection, TestCase};

use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Clone, PartialEq, Eq)]
enum TestCaseSelector {
    Number(usize),
    Name(String),
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    file: PathBuf,
    test: Option<TestCaseSelector>,
    #[arg(long, short)]
    output: Option<PathBuf>,
    #[arg(long, short, value_parser = parse_timescale)]
    timescale: Option<String>,
    #[arg(long)]
    default: bool,
    #[arg(long, short, default_value = "10:0", value_parser = parse_delay)]
    delay: (u32, u32),
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum VerilogValue {
    Value(i64),
    Z,
}

struct VerilogIdentifier<'a> {
    identifier: &'a str,
    suffix: Option<&'a str>,
}

impl From<OutputValue> for VerilogValue {
    fn from(value: OutputValue) -> Self {
        match value {
            OutputValue::Value(num) => VerilogValue::Value(num),
            OutputValue::Z => VerilogValue::Z,
            OutputValue::X => panic!("Unexpected X output value"),
        }
    }
}

impl<'a> From<&'a str> for VerilogIdentifier<'a> {
    fn from(value: &'a str) -> Self {
        VerilogIdentifier {
            identifier: value,
            suffix: None,
        }
    }
}

impl<'a> From<&'a String> for VerilogIdentifier<'a> {
    fn from(value: &'a String) -> Self {
        VerilogIdentifier {
            identifier: value.as_str(),
            suffix: None,
        }
    }
}

impl<'a> VerilogIdentifier<'a> {
    fn with_suffix(identifier: &'a str, suffix: &'a str) -> Self {
        Self {
            identifier,
            suffix: Some(suffix),
        }
    }
}

impl From<InputValue> for VerilogValue {
    fn from(value: InputValue) -> Self {
        match value {
            InputValue::Value(num) => VerilogValue::Value(num),
            InputValue::Z => VerilogValue::Z,
        }
    }
}

impl std::fmt::Display for VerilogValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerilogValue::Value(num) => write!(f, "{num}"),
            VerilogValue::Z => write!(f, "'Z"),
        }
    }
}

impl<'a> std::fmt::Display for VerilogIdentifier<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\\{}{} ", self.identifier, self.suffix.unwrap_or(""))
    }
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
    out: &mut Box<dyn std::io::Write>,
    inputs: impl Iterator<Item = DataEntry<'a, InputValue>>,
    outputs: impl Iterator<Item = DataEntry<'a, OutputValue>>,
    bidirectional: &[&str],
    delay: (u32, u32),
) -> anyhow::Result<()> {
    for input in inputs {
        if bidirectional.contains(&input.name) {
            writeln!(
                out,
                "    {}= {};",
                VerilogIdentifier::with_suffix(input.name, "_reg"),
                VerilogValue::from(input.value)
            )?;
        } else {
            writeln!(
                out,
                "    {}= {};",
                VerilogIdentifier::from(input.name),
                VerilogValue::from(input.value)
            )?;
        }
    }
    writeln!(out, "#{};", delay.0)?;
    for output in outputs {
        writeln!(
            out,
            "    `assert_eq({}, {});",
            VerilogIdentifier::from(output.name),
            VerilogValue::from(output.value)
        )?;
    }
    if delay.1 > 0 {
        writeln!(out, "#{};", delay.1)?;
    }
    writeln!(out)?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let path = cli.file;
    eprintln!("Loading {path:?}");
    let input = std::fs::read_to_string(&path).unwrap();
    let dig_file = dig::parse(&input).unwrap();

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
                anyhow::bail!("No test case \"{name}\" found");
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
                anyhow::bail!("Please specify a test case");
            }
        }
    };
    eprintln!("Loading test case #{test_num}");
    let test_case = TestCase::try_from_static_dig(&dig_file, test_num)?;

    let mut out: Box<dyn std::io::Write> = if let Some(path) = cli.output {
        let Ok(file) = std::fs::File::create(&path) else {
            anyhow::bail!("Could not open file {path:?} for output");
        };
        eprintln!("Writing output to {path:?}");
        Box::new(file)
    } else {
        Box::new(std::io::stdout())
    };

    if let Some(timescale) = cli.timescale {
        writeln!(out, "`timescale {timescale}\n")?;
    }

    writeln!(
        out,
        r#"`define assert_eq(signal, value) \
    if (signal !== value) begin \
        $display("ASSERTION FAILED in %m: signal != value"); \
        error_count += 1; \
    end"#
    )?;
    writeln!(out,)?;

    let bidirectional: Vec<&str> = {
        let outputs: Vec<_> = test_case
            .signals
            .iter()
            .filter_map(|sig| {
                if sig.is_output() {
                    Some(&sig.name)
                } else {
                    None
                }
            })
            .collect();

        test_case
            .signals
            .iter()
            .filter_map(|sig| {
                if sig.is_input() && outputs.contains(&&sig.name) {
                    Some(sig.name.as_str())
                } else {
                    None
                }
            })
            .collect()
    };

    let ports = test_case
        .signals
        .iter()
        .filter_map(|sig| {
            let io_type = match sig.dir {
                SignalDirection::Input { .. } => {
                    if bidirectional.contains(&sig.name.as_str()) {
                        "inout"
                    } else {
                        "output reg"
                    }
                }
                SignalDirection::Output => {
                    if bidirectional.contains(&sig.name.as_str()) {
                        return None;
                    } else {
                        "input"
                    }
                }
            };
            let width = if sig.bits > 1 {
                format!("[{}:0]", sig.bits - 1)
            } else {
                String::from("")
            };
            Some(format!(
                "    {io_type} {width} {}",
                VerilogIdentifier::from(&sig.name),
            ))
        })
        .collect::<Vec<_>>()
        .join(",\n");
    writeln!(out, "module tb (\n{ports}\n);")?;
    writeln!(out, "integer error_count = 0;")?;
    for name in &bidirectional {
        writeln!(
            out,
            "reg {}= 1'bZ;",
            VerilogIdentifier::with_suffix(name, "_reg")
        )?;
    }

    for name in &bidirectional {
        writeln!(
            out,
            "assign {}= {};",
            VerilogIdentifier::from(*name),
            VerilogIdentifier::with_suffix(name, "_reg")
        )?;
    }
    writeln!(out, "initial begin")?;

    if cli.default {
        let row = test_case.default_row();
        print_row(
            &mut out,
            row.inputs(),
            row.checked_outputs(),
            &bidirectional,
            cli.delay,
        )?;
    }

    for row in test_case.iter() {
        print_row(
            &mut out,
            row.changed_inputs(),
            row.checked_outputs(),
            &bidirectional,
            cli.delay,
        )?;
    }

    writeln!(out, "  if(error_count > 0) begin")?;
    writeln!(out, "    $display(\"There were failed assertions\");")?;
    writeln!(out, "    $finish_and_return(1);")?;
    writeln!(out, "  end")?;
    writeln!(out, "  $display(\"All tests passed.\");")?;

    writeln!(out, "end")?;
    writeln!(out, "endmodule")?;

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
