use std::collections::VecDeque;
use crate::common::{Context, InputProvider};
use anyhow::Context as AnyhowContext;
use std::str::FromStr;

pub fn run(context: &mut Context) {
    context.add_test_inputs(get_test_inputs());

    let input = context.get_input();
    let input = input.as_str();

    run1(input, None);

    solve(input);

    run1(input, Some(190384609508367));
}

fn run1(input: &str, override_a: Option<u64>) {
    let mut parts = input.split("\n\n");

    let mut program = Program {
        registers: parts.next().unwrap().parse().unwrap(),
        output: Default::default(),
        program_counter: 0,
    };

    let instructions: Vec<Instruction> = parts
        .next()
        .unwrap()
        .split_whitespace()
        .nth(1)
        .unwrap()
        .split(",")
        .map(|n| n.parse())
        .collect::<Result<_, _>>()
        .unwrap();

    if let Some(override_a) = override_a {
        program.registers.a = override_a;
    }
    log::trace!("initial: {:?}", program);
    while program.next(&instructions) {
        log::trace!("{:?}", program);
    }

    println!("output: {}", program.get_output());
}

fn solve(input: &str) {
    let mut parts = input.split("\n\n");

    let program = Program {
        registers: parts.next().unwrap().parse().unwrap(),
        output: Default::default(),
        program_counter: 0,
    };

    let instructions_str = parts.next().unwrap().split_whitespace().nth(1).unwrap();

    let instructions: Vec<Instruction> = instructions_str
        .split(",")
        .map(|n| n.parse())
        .collect::<Result<_, _>>()
        .unwrap();

    let solve_for_a = |out_values: String, a_aggregate: u64| {
        for a in 0..8u64.pow(5) {
            let mut program = program.clone();
            program.registers.a = a + a_aggregate;
            while program.next(&instructions) {}
            if program.get_output() == out_values {
                log::debug!(
                    "target={} got a={}, a + aggregate ={} outputs = {}",
                    out_values,
                    a,
                    a + a_aggregate,
                    program.get_output()
                );
                return a;
            }
        }
        panic!("could not find a for {} :(", out_values);
    };

    let mut a = 0u64;
    let mut out_values = VecDeque::new();
    for instruction in instructions.iter().rev() {
        a *= 8;
        out_values.push_front(instruction.0);
        let out_values = out_values
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let a_diff = solve_for_a(out_values, a);
        a += a_diff;
    }

    println!("lowest A that solves: {}", a);
}

#[derive(Debug, Clone)]
struct Program {
    registers: Registers,
    output: Vec<u64>,
    program_counter: usize,
}

impl Program {
    pub fn get_output(&self) -> String {
        self.output
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(",")
    }
    pub fn next(&mut self, instructions: &[Instruction]) -> bool {
        if self.program_counter >= instructions.len() {
            return false;
        }
        let instruction = &instructions[self.program_counter];
        let operand = &instructions[self.program_counter + 1];
        log::trace!("{}({})", Operations::get_name(instruction.0), operand.0);
        if let Some(jump) =
            instruction.as_operation()(&mut self.registers, operand, &mut self.output)
        {
            self.program_counter = jump;
        } else {
            self.program_counter += 2;
        }
        true
    }
}

#[derive(Debug, Clone)]
struct Registers {
    a: u64,
    b: u64,
    c: u64,
}

#[derive(Clone)]
struct Instruction(u64);

impl Instruction {
    pub fn as_operation(&self) -> &'static OperationInputs {
        Operations::get(self.0)
    }
    pub fn as_literal(&self) -> u64 {
        self.0
    }
    pub fn as_combo(&self, registers: &Registers) -> u64 {
        match self.0 {
            0..=3 => self.0,
            4 => registers.a,
            5 => registers.b,
            6 => registers.c,
            7 => panic!("invalid 7"),
            other => panic!("instruction out of range: {}", other),
        }
    }
}

type OperationInputs = dyn Fn(&mut Registers, &Instruction, &mut Vec<u64>) -> Option<usize>;

struct Operations();
impl Operations {
    pub fn get_name(opcode: u64) -> &'static str {
        match opcode {
            0 => "adv",
            1 => "bxl",
            2 => "bst",
            3 => "jnz",
            4 => "bxc",
            5 => "out",
            6 => "bdv",
            7 => "cdv",
            _ => "invalid opcode",
        }
    }
    pub fn get(opcode: u64) -> &'static OperationInputs {
        match opcode {
            0 => Self::adv(),
            1 => Self::bxl(),
            2 => Self::bst(),
            3 => Self::jnz(),
            4 => Self::bxc(),
            5 => Self::out(),
            6 => Self::bdv(),
            7 => Self::cdv(),
            other => panic!("invalid opcode {}", other),
        }
    }
    #[inline]
    pub fn adv() -> &'static OperationInputs {
        &|registers, operand, _output| {
            let denominator = 2u64.pow(operand.as_combo(registers) as u32);
            registers.a /= denominator;

            None
        }
    }
    #[inline]
    pub fn bxl() -> &'static OperationInputs {
        &|registers, operand, _output| {
            registers.b ^= operand.as_literal();
            None
        }
    }
    #[inline]
    pub fn bst() -> &'static OperationInputs {
        &|registers, operand, _output| {
            registers.b = operand.as_combo(registers) % 8;
            None
        }
    }
    #[inline]
    pub fn jnz() -> &'static OperationInputs {
        &|registers, operand, _output| {
            if registers.a == 0 {
                None
            } else {
                Some(operand.as_literal() as usize)
            }
        }
    }
    #[inline]
    pub fn bxc() -> &'static OperationInputs {
        &|registers, _operand, _output| {
            registers.b ^= registers.c;
            None
        }
    }
    #[inline]
    pub fn out() -> &'static OperationInputs {
        &|registers, operand, output| {
            output.push(operand.as_combo(registers) % 8);
            None
        }
    }
    #[inline]
    pub fn bdv() -> &'static OperationInputs {
        &|registers, operand, _output| {
            let denominator = 2u64.pow(operand.as_combo(registers) as u32);
            registers.b = registers.a / denominator;
            None
        }
    }
    #[inline]
    pub fn cdv() -> &'static OperationInputs {
        &|registers, operand, _output| {
            let denominator = 2u64.pow(operand.as_combo(registers) as u32);
            registers.c = registers.a / denominator;
            None
        }
    }
}

impl FromStr for Instruction {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

impl FromStr for Registers {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.lines();
        let mut get_next = || {
            parts
                .next()
                .context("not enough register lines")
                .and_then(|line| line.split_whitespace().last().context("empty line"))
                .and_then(|word| word.parse::<u64>().context("register parse fail"))
        };

        Ok(Self {
            a: get_next()?,
            b: get_next()?,
            c: get_next()?,
        })
    }
}

fn get_test_inputs() -> impl Iterator<Item = Box<InputProvider>> {
    [
        "Register A: 729
Register B: 0
Register C: 0

Program: 0,1,5,4,3,0",
        "Register A: 2024
Register B: 0
Register C: 0

Program: 0,3,5,4,3,0",
    ]
    .into_iter()
    .map(|input| Box::new(move || input.into()) as Box<InputProvider>)
}
