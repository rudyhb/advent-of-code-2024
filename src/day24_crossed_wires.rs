use crate::common::{Context, InputProvider};
use anyhow::Context as AnyhowContext;
use colored::*;
use itertools::Itertools;
use once_cell::sync::Lazy;
use regex::Regex;
use std::borrow::Cow;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

pub fn run(context: &mut Context) {
    context.add_test_inputs(get_test_inputs());

    let input = context.get_input();
    let input = input.as_str();

    let mut input = input.split("\n\n");
    let inputs: Vec<Wire> = input
        .next()
        .unwrap()
        .lines()
        .map(|line| line.parse())
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let gates: Vec<Gate> = input
        .next()
        .unwrap()
        .lines()
        .map(|line| line.parse())
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let mut circuit = Circuit::new(inputs.clone().into_iter(), gates.clone().into_iter());
    println!("output: {}", circuit.get_output("z").unwrap());

    let
        //mut 
        circuit = Circuit::new(inputs.clone().into_iter(), gates.clone().into_iter());

    // FIXES
    //circuit.swap_bits("hbs", "kfp");
    //circuit.swap_bits("z18", "dhq");
    //circuit.swap_bits("z22", "pdg");
    //circuit.swap_bits("z27", "jcp");

    print_circuit(&circuit, 46);

    let mut password = ["hbs", "kfp", "z18", "dhq", "z22", "pdg", "z27", "jcp"];
    password.sort();
    println!("password: {}", password.join(","));
}

fn print_circuit(circuit: &Circuit, output_bits: usize) {
    println!("full adder circuit:\n");

    for i in 0..output_bits {
        let s = format_element(circuit, &format!("z{:02}", i));
        println!("z{:02}: {}", i, s);

        let mut out_of_place = vec![];
        for should_not in (i + 1..output_bits).flat_map(|i| {
            ['y', 'x']
                .into_iter()
                .map(move |x| format!("{}{:02}", x, i))
        }) {
            if s.contains(&should_not) {
                out_of_place.push(format!("SHOULDN'T '{}'", should_not));
            }
        }
        for should_contain_one in (1..=i.min(output_bits - 2))
            .map(|i| {
                [
                    format!("x{:02} XOR y{:02}", i, i),
                    format!("y{:02} XOR x{:02}", i, i),
                ]
            })
            .chain((0..i).map(|i| {
                [
                    format!("x{:02} AND y{:02}", i, i),
                    format!("y{:02} AND x{:02}", i, i),
                ]
            }))
        {
            let count = s.split(&should_contain_one[0]).count() - 1
                + s.split(&should_contain_one[1]).count()
                - 1;
            if count != 1 {
                out_of_place.push(format!("COUNT={} '{}'", count, should_contain_one[0]))
            }
        }

        let words: HashMap<&str, usize> = s
            .split(['(', ')'])
            .map(|w| w.trim())
            .filter(|w| !w.is_empty())
            .fold(HashMap::new(), |mut acc, next| {
                *acc.entry(next).or_default() += 1;
                acc
            });

        let xor = words.get("XOR").copied().unwrap_or_default();
        let and = words.get("AND").copied().unwrap_or_default();
        let or = words.get("OR").copied().unwrap_or_default();
        if xor != 1.min(i) && i != output_bits - 1 {
            out_of_place.push(format!("XOR COUNT={}", xor));
        }
        if and != i.saturating_sub(1) {
            out_of_place.push(format!("AND COUNT={}", and));
        }
        if or != i.saturating_sub(1) {
            out_of_place.push(format!("OR COUNT={}", and));
        }

        let try_to_num = |s: &str| -> Option<usize> {
            let s = s.split_whitespace().next()?;
            let s: String = s.chars().filter(|c| c.is_numeric()).collect();
            s.parse().ok()
        };
        let comp_numbers = |a: &str, b: &str| -> Ordering {
            let a = if let Some(a) = try_to_num(a) {
                a
            } else {
                return Ordering::Less;
            };
            let b = if let Some(b) = try_to_num(b) {
                b
            } else {
                return Ordering::Greater;
            };

            a.cmp(&b)
        };
        let words: Vec<_> = words
            .into_iter()
            .sorted_by(|(a, _), (b, _)| {
                let comp = comp_numbers(a, b);
                if let Ordering::Equal = comp {
                    a.cmp(b)
                } else {
                    comp
                }
            })
            .collect();
        println!("{:?}", words);

        if !out_of_place.is_empty() {
            println!("{}{}", "ERRORS: ".red(), out_of_place.join(" | "));
        }

        println!("\n");
    }
}

fn format_element(circuit: &Circuit, element: &str) -> String {
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[xy]\d+").expect("invalid regex"));
    if RE.is_match(element) {
        element.to_string()
    } else {
        let gate = circuit
            .gates
            .get(element)
            .with_context(|| format!("cannot find gate for output '{}'", element))
            .unwrap();
        let a = format_element(circuit, &gate.a);
        let b = format_element(circuit, &gate.b);
        format!("( {} {} {} )", a, gate.operation, b)
    }
}

#[derive(Clone)]
struct Circuit {
    wires: WireCircuit,
    gates: HashMap<Cow<'static, str>, Gate>,
}

impl Circuit {
    #[allow(unused)]
    pub fn swap_bits(&mut self, left: &str, right: &str) {
        let (left, left_value) = self
            .gates
            .remove_entry(left)
            .with_context(|| format!("swap_bits: cannot find {}", left))
            .unwrap();
        let (right, right_value) = self
            .gates
            .remove_entry(right)
            .with_context(|| format!("swap_bits: cannot find {}", right))
            .unwrap();
        self.gates.insert(left, right_value);
        self.gates.insert(right, left_value);
    }
}

#[derive(Clone)]
struct WireCircuit(HashMap<Cow<'static, str>, bool>);

impl Debug for WireCircuit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for (name, _) in self.0.iter().filter(|(_, val)| **val) {
            write!(f, "{}, ", name)?;
        }
        write!(f, "]")
    }
}

#[derive(thiserror::Error, Debug)]
enum WireCircuitError {
    #[error("circular access")]
    CircularAccess,
}

impl WireCircuit {
    pub fn get(
        &mut self,
        wire_name: &str,
        gates: &HashMap<Cow<'static, str>, Gate>,
        mut history: HashSet<String>,
    ) -> Result<Option<bool>, WireCircuitError> {
        if let Some(&value) = self.0.get(wire_name) {
            return Ok(Some(value));
        }
        if history.contains(wire_name) {
            return Err(WireCircuitError::CircularAccess);
        }
        history.insert(wire_name.to_owned());

        let gate = if let Some(gate) = gates.get(wire_name) {
            gate
        } else {
            return Ok(None);
        };
        let a = self
            .get(&gate.a, gates, history.clone())?
            .with_context(|| format!("cannot find a for gate {:?}", gate))
            .unwrap();
        let b = self
            .get(&gate.b, gates, history)?
            .with_context(|| format!("cannot find b for gate {:?}", gate))
            .unwrap();
        let output = gate.operation.operate(a, b);
        self.0.insert(wire_name.to_owned().into(), output);
        Ok(Some(output))
    }
}

impl Circuit {
    pub fn new(wires: impl Iterator<Item = Wire>, gates: impl Iterator<Item = Gate>) -> Self {
        Self {
            wires: WireCircuit(
                wires
                    .map(|wire| (Cow::Owned(wire.name), wire.value))
                    .collect(),
            ),
            gates: gates
                .map(|gate| (Cow::Owned(gate.output.clone()), gate))
                .collect(),
        }
    }
    pub fn get_output(&mut self, prefix: &'static str) -> Result<u64, WireCircuitError> {
        let mut result = 0u64;
        for i in 0.. {
            let name = Cow::Owned(format!("{}{:02}", prefix, i));
            if let Some(output) = self.wires.get(&name, &self.gates, Default::default())? {
                if output {
                    result += 2u64.pow(i);
                }
            } else {
                break;
            }
        }
        Ok(result)
    }
}

#[derive(Clone)]
struct Wire {
    name: String,
    value: bool,
}

#[derive(Debug, Clone)]
struct Gate {
    operation: Operation,
    a: Cow<'static, str>,
    b: Cow<'static, str>,
    output: String,
}

#[derive(Debug, Clone, Copy)]
enum Operation {
    And,
    Or,
    Xor,
}

impl Operation {
    pub fn operate(&self, a: bool, b: bool) -> bool {
        match self {
            Operation::And => a && b,
            Operation::Or => a || b,
            Operation::Xor => a ^ b,
        }
    }
}

impl FromStr for Wire {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split(':');
        let name = parts.next().context("empty string")?.trim().to_string();
        let value = parts.next().context("no value found")?.trim() == "1";
        Ok(Self { name, value })
    }
}

impl FromStr for Gate {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split_whitespace();
        let a = parts.next().context("a")?.to_string().into();
        let operation = match parts.next().context("OP")? {
            "XOR" => Operation::Xor,
            "OR" => Operation::Or,
            "AND" => Operation::And,
            other => return Err(anyhow::anyhow!("invalid operation {}", other)),
        };
        let b = parts.next().context("b")?.to_string().into();
        let output = parts.nth(1).context("output")?.to_string();
        Ok(Self {
            operation,
            a,
            b,
            output,
        })
    }
}

impl Display for Gate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {} -> {}",
            self.a, self.operation, self.b, self.output
        )
    }
}

impl Display for Operation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Operation::And => write!(f, "AND"),
            Operation::Or => write!(f, "OR"),
            Operation::Xor => write!(f, "XOR"),
        }
    }
}

fn get_test_inputs() -> impl Iterator<Item = Box<InputProvider>> {
    [
        "x00: 0
x01: 1
x02: 0
x03: 1
x04: 0
x05: 1
y00: 0
y01: 0
y02: 1
y03: 1
y04: 0
y05: 1

x00 AND y00 -> z05
x01 AND y01 -> z02
x02 AND y02 -> z01
x03 AND y03 -> z03
x04 AND y04 -> z04
x05 AND y05 -> z00",
        "x00: 1
x01: 0
x02: 1
x03: 1
x04: 0
y00: 1
y01: 1
y02: 1
y03: 1
y04: 1

ntg XOR fgs -> mjb
y02 OR x01 -> tnw
kwq OR kpj -> z05
x00 OR x03 -> fst
tgd XOR rvg -> z01
vdt OR tnw -> bfw
bfw AND frj -> z10
ffh OR nrd -> bqk
y00 AND y03 -> djm
y03 OR y00 -> psh
bqk OR frj -> z08
tnw OR fst -> frj
gnj AND tgd -> z11
bfw XOR mjb -> z00
x03 OR x00 -> vdt
gnj AND wpb -> z02
x04 AND y00 -> kjc
djm OR pbm -> qhw
nrd AND vdt -> hwm
kjc AND fst -> rvg
y04 OR y02 -> fgs
y01 AND x02 -> pbm
ntg OR kjc -> kwq
psh XOR fgs -> tgd
qhw XOR tgd -> z09
pbm OR djm -> kpj
x03 XOR y03 -> ffh
x00 XOR y04 -> ntg
bfw OR bqk -> z06
nrd XOR fgs -> wpb
frj XOR qhw -> z04
bqk OR frj -> z07
y03 OR x01 -> nrd
hwm AND bqk -> z03
tgd XOR rvg -> z12
tnw OR pbm -> gnj",
    ]
    .into_iter()
    .map(|input| Box::new(move || input.into()) as Box<InputProvider>)
}
