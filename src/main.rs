use digital_test_runner::dig;

use clap::Parser;
use std::path::PathBuf;

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

fn main() -> miette::Result<()> {
    let cli = Cli::parse();

    let path = cli.file;
    eprintln!("Loading {path:?}");
    let dig_file = dig::File::open(&path)?;

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
                        eprintln!("{i}: (unnamed)");
                    } else {
                        eprintln!("{i}: {}", test_case.name);
                    }
                }
                miette::bail!("Please specify a test case");
            }
        }
    };

    eprintln!(
        "Loading test case #{test_num}: {}",
        dig_file.test_cases[test_num].name
    );
    let test_case = dig_file.load_test(test_num)?;

    let builder = digital_test_to_verilog::Builder::try_new(&test_case)?;

    if let Some(path) = &cli.output {
        eprintln!("Writing output to {path:?}");
    }

    builder
        .with_delay(cli.delay)
        .with_timescale(cli.timescale)
        .with_output(cli.output)
        .done()
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
