use digital_test_runner::{dig, SignalDirection, TestCase};

fn main() -> anyhow::Result<()> {
    let path = std::env::args().nth(1).unwrap_or(String::from("ALU.dig"));
    eprintln!("Loading {path}");
    let input = std::fs::read_to_string(path).unwrap();
    let dig_file = dig::parse(&input).unwrap();

    let test_case = TestCase::try_from_static_dig(&dig_file, 1)?;

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

    for row in &test_case {
        for input in row.changed_inputs() {
            println!("    \\{} = {};", input.name, input.value);
        }
        println!("#10;");
        for output in row.checked_outputs() {
            println!("    `assert_eq(\\{} , {});", output.name, output.value);
        }
        println!("#10;");
        println!();
    }

    println!("end");
    println!("endmodule");

    Ok(())
}
