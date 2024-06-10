use digital_test_runner::{InputValue, OutputValue, Signal};

const REG_SUFFIX: &str = "_reg";

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub(crate) enum VerilogValue {
    Value(i64),
    Z,
}

pub(crate) struct VerilogIdentifier<'a> {
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

impl<'a> From<&'a Signal> for VerilogIdentifier<'a> {
    fn from(signal: &'a Signal) -> Self {
        VerilogIdentifier {
            identifier: signal.name.as_str(),
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

    pub(crate) fn from_input(signal: &'a Signal) -> Self {
        let name = signal.name.as_str();
        if signal.is_bidirectional() {
            VerilogIdentifier::with_suffix(name, REG_SUFFIX)
        } else {
            VerilogIdentifier::from(name)
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
        static RE: once_cell::sync::Lazy<regex::Regex> =
            once_cell::sync::Lazy::new(|| regex::Regex::new(r"^[a-zA-Z_][a-zA-Z0-9$_]*$").unwrap());

        if RE.is_match(&self.identifier) && self.suffix.map(|s| RE.is_match(s)).unwrap_or(true) {
            write!(f, "{}{}", self.identifier, self.suffix.unwrap_or(""))
        } else {
            write!(f, "\\{}{} ", self.identifier, self.suffix.unwrap_or(""))
        }
    }
}
