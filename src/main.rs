use digital_test_runner::{dig, TestCaseLoader};

fn main() -> anyhow::Result<()> {
    let path = std::env::args().nth(1).unwrap_or(String::from("ALU.dig"));
    eprintln!("Loading {path}");
    let input = std::fs::read_to_string(path).unwrap();
    let dig_file = dig::parse(&input).unwrap();
    let mut builder = TestCaseLoader::try_from_dig(&dig_file, 2)?;
    for input in dig_file.inputs.iter().filter(|input| input.bits > 1) {
        builder = builder.expand(&input.name, input.bits);
    }
    for output in dig_file.outputs.iter().filter(|output| output.bits > 1) {
        builder = builder.expand(&output.name, output.bits);
    }
    let test_case = builder.try_build()?;
    let results = test_case.run();

    println!(
        r#"`define assert_eq(signal, value) \
    if (signal !== value) begin \
        $display("ASSERTION FAILED in %m: signal != value"); \
    end"#
    );
    println!();

    println!("module tb (");

    let mut ports = Vec::with_capacity(test_case.inputs.len() + test_case.outputs.len());
    ports.extend(
        test_case
            .inputs
            .iter()
            .map(|input| format!("    output reg \\{} ", input.name)),
    );
    ports.extend(
        test_case
            .outputs
            .iter()
            .map(|output| format!("    input \\{} ", output.name)),
    );
    println!("{}", ports.join(",\n"));
    println!(");");
    println!("initial begin");
    for input in &test_case.inputs {
        let val = match input.default {
            digital_test_runner::InputValue::Value(n) => format!("{n}"),
            digital_test_runner::InputValue::Z => String::from("Z"),
        };
        println!("    \\{} = {val};", input.name);
    }
    println!("#10;");
    println!("#10;");

    let mut prev_data = test_case
        .inputs
        .iter()
        .map(|input| match input.default {
            digital_test_runner::InputValue::Value(n) => digital_test_runner::DataResult::Number(n),
            digital_test_runner::InputValue::Z => digital_test_runner::DataResult::Z,
        })
        .collect::<Vec<_>>();

    for result in results {
        for (input, (data, prev_data)) in test_case
            .inputs
            .iter()
            .zip(result.inputs.iter().zip(prev_data.iter_mut()))
        {
            if data == prev_data {
                continue;
            }
            let val = match data {
                digital_test_runner::DataResult::Number(n) => format!("{n}"),
                digital_test_runner::DataResult::X => unreachable!(),
                digital_test_runner::DataResult::Z => String::from("Z"),
            };
            println!("    \\{} = {val};", input.name);
            *prev_data = data.clone();
        }
        println!("#10;");
        for (output, data) in test_case.outputs.iter().zip(result.outputs.iter()) {
            if let digital_test_runner::DataResult::Number(n) = data {
                let n = n & (1 << output.bits - 1);
                println!("    `assert_eq(\\{} , {n});", output.name);
            }
        }
        println!("#10;");
        println!();
    }

    println!("end");
    println!("endmodule");

    Ok(())
}
