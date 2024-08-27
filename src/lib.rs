use digital_test_runner::{
    static_test::StaticDataRowIterator, ExpectedEntry, ExpectedValue, InputEntry, InputValue,
    SignalType, TestCase,
};
use miette::IntoDiagnostic;
use verilog::{VerilogIdentifier, VerilogValue};

mod verilog;

pub struct Builder<'a> {
    test_case: &'a TestCase,
    it: StaticDataRowIterator<'a>,
    output_path: Option<std::path::PathBuf>,
    timescale: Option<String>,
    delay: Option<(u32, u32)>,
}

impl<'a> Builder<'a> {
    pub fn try_new(test_case: &'a TestCase) -> miette::Result<Self> {
        let it = test_case.try_iter_static()?;
        Ok(Self {
            test_case,
            it,
            output_path: None,
            timescale: None,
            delay: None,
        })
    }

    pub fn with_output(mut self, path: impl Into<Option<std::path::PathBuf>>) -> Self {
        self.output_path = path.into();
        self
    }

    pub fn with_timescale(mut self, timescale: impl Into<Option<String>>) -> Self {
        self.timescale = timescale.into();
        self
    }

    pub fn with_delay(mut self, delay: impl Into<Option<(u32, u32)>>) -> Self {
        self.delay = delay.into();
        self
    }

    pub fn done(self) -> miette::Result<()> {
        output_verilog(
            self.test_case,
            self.it,
            self.output_path,
            self.timescale,
            self.delay.unwrap_or((0, 10)),
        )
    }
}

macro_rules! outputln {
    ($($t:tt)*) => {{
        writeln!($($t)*).into_diagnostic()
    }};
}

fn print_row<'a, Out: std::io::Write>(
    line: usize,
    out: &mut Out,
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

fn output_verilog(
    test_case: &TestCase,
    it: StaticDataRowIterator,
    path: Option<std::path::PathBuf>,
    timescale: Option<String>,
    delay: (u32, u32),
) -> miette::Result<()> {
    let mut out: Box<dyn std::io::Write> = if let Some(path) = path {
        let Ok(file) = std::fs::File::create(&path) else {
            miette::bail!("Could not open file {path:?} for output");
        };
        eprintln!("Writing output to {path:?}");
        Box::new(file)
    } else {
        Box::new(std::io::stdout())
    };
    let out = &mut out;

    if let Some(timescale) = timescale {
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
            out,
            row.inputs.iter().filter(|inp| inp.changed),
            row.expected
                .iter()
                .filter(|exp| exp.value != ExpectedValue::X),
            delay,
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
