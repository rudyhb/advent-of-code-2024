use crate::common::{Context, InputProvider};
use std::str;

pub fn run(context: &mut Context) {
    context.add_test_inputs(get_test_inputs());

    let input = context.get_input();
    let input = input.as_str();

    let operations = parse_operations(input);
    let sum: i32 = operations.iter().map(|o| o.product()).sum();
    println!("Total sum of products: {}", sum);

    let product = calculate_sum_products(input);
    println!("Product v2: {}", product);
}

fn parse_operations(input: &str) -> Vec<MultiplicationOperation> {
    let mut result = Vec::new();

    let mut i = 0usize;
    let input = input.as_bytes();
    while i < input.len() {
        let input = &input[i..];

        match input.into() {
            MultiplicationOperationParsing::Success(MultiplicationOperationParsed {
                operation,
                str_len,
            }) => {
                result.push(operation);
                i += str_len;
            }
            MultiplicationOperationParsing::Failure { first_char_len } => {
                i += first_char_len;
            }
        }
    }

    result
}

fn calculate_sum_products(input: &str) -> i32 {
    let mut sum = 0i32;

    let mut enabled = true;
    let mut i = 0usize;
    let input = input.as_bytes();
    while i < input.len() {
        let input = &input[i..];

        if let Ok(operation) = DoDontOperation::try_from(input) {
            match operation {
                DoDontOperation::Do { str_len } => {
                    log::debug!("do() operation found, enabling");
                    enabled = true;
                    i += str_len;
                }
                DoDontOperation::Dont { str_len } => {
                    log::debug!("don't() operation found, disabling");
                    enabled = false;
                    i += str_len;
                }
            }
            continue;
        }

        match input.into() {
            MultiplicationOperationParsing::Success(MultiplicationOperationParsed {
                operation,
                str_len,
            }) => {
                log::debug!(
                    "product mul({},{}) = {} will{} be counted",
                    operation.left,
                    operation.right,
                    operation.product(),
                    if enabled { "" } else { " NOT" }
                );
                if enabled {
                    sum += operation.product();
                }
                i += str_len;
            }
            MultiplicationOperationParsing::Failure { first_char_len } => {
                i += first_char_len;
            }
        }
    }

    sum
}

struct MultiplicationOperation {
    left: i32,
    right: i32,
}

impl MultiplicationOperation {
    fn product(&self) -> i32 {
        self.left * self.right
    }
}

enum MultiplicationOperationParsing {
    Success(MultiplicationOperationParsed),
    Failure { first_char_len: usize },
}

struct MultiplicationOperationParsed {
    operation: MultiplicationOperation,
    str_len: usize,
}

enum DoDontOperation {
    Do { str_len: usize },
    Dont { str_len: usize },
}

impl From<&[u8]> for MultiplicationOperationParsing {
    fn from(value: &[u8]) -> Self {
        const START_LEN: usize = "mul(".len();
        const MID_LEN: usize = ",".len();
        const END_LEN: usize = ")".len();

        log::trace!("Parsing: {}", str::from_utf8(value).unwrap());

        let first_char_len: usize;
        unsafe {
            first_char_len = str::from_utf8_unchecked(value)
                .chars()
                .next()
                .unwrap()
                .len_utf8();
        }

        if value.len() < START_LEN + MID_LEN + END_LEN + 2 {
            log::trace!("Value too short: {}", str::from_utf8(value).unwrap());
            return Self::Failure { first_char_len };
        }

        let left: i32;
        let right: i32;
        let mut str_len = START_LEN;

        if &value[..START_LEN] != b"mul(" {
            log::trace!(
                "Not a mul operation: {}",
                str::from_utf8(&value[..START_LEN]).unwrap()
            );
            return Self::Failure { first_char_len };
        }

        let mut value = &value[START_LEN..];
        if let Some(pos) = value.iter().position(|c| *c == b',') {
            let left_value: Result<i32, _>;
            unsafe {
                left_value = str::from_utf8_unchecked(&value[..pos]).parse();
            }
            match left_value {
                Ok(val) => {
                    left = val;
                }
                Err(e) => {
                    log::trace!("Failed to parse left value: {}", e);
                    return Self::Failure { first_char_len };
                }
            }
            value = &value[pos + MID_LEN..];
            str_len += pos + MID_LEN;
        } else {
            log::trace!(
                "No comma found for op mul({}",
                str::from_utf8(&value).unwrap()
            );
            return Self::Failure { first_char_len };
        }

        if let Some(pos) = value.iter().position(|c| *c == b')') {
            let right_value: Result<i32, _>;
            unsafe {
                right_value = str::from_utf8_unchecked(&value[..pos]).parse();
            }
            match right_value {
                Ok(val) => {
                    right = val;
                }
                Err(e) => {
                    log::trace!("Failed to parse right value: {}", e);
                    return Self::Failure { first_char_len };
                }
            }
            str_len += pos + END_LEN;
        } else {
            log::trace!(
                "No ) found for op mul({},{}",
                left,
                str::from_utf8(&value).unwrap()
            );
            return Self::Failure { first_char_len };
        }

        log::trace!("Parsed: mul({},{}) = {}", left, right, left * right);

        Self::Success(MultiplicationOperationParsed {
            operation: MultiplicationOperation { left, right },
            str_len,
        })
    }
}

impl TryFrom<&[u8]> for DoDontOperation {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        const DO_LEN: usize = "do()".len();
        const DONT_LEN: usize = "don't()".len();

        if value.starts_with(b"do()") {
            Ok(DoDontOperation::Do { str_len: DO_LEN })
        } else if value.starts_with(b"don't()") {
            Ok(DoDontOperation::Dont { str_len: DONT_LEN })
        } else {
            Err(anyhow::anyhow!("Not a do or dont operation"))
        }
    }
}

fn get_test_inputs() -> impl Iterator<Item = Box<InputProvider>> {
    [
        "xmul(2,4)%&mul[3,7]!@^do_not_mul(5,5)+mul(32,64]then(mul(11,8)mul(8,5))",
        "xmul(2,4)&mul[3,7]!^don't()_mul(5,5)+mul(32,64](mul(11,8)undo()?mul(8,5))",
    ]
    .into_iter()
    .map(|input| Box::new(move || input.into()) as Box<InputProvider>)
}
