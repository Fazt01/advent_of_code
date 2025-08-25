use anyhow::{bail, Context, Ok, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use std::io;
use std::io::Read;

static RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(concat!(
        r"Monkey \d+:\s+",
        r"  Starting items: ([\d ,]+)\s+",
        r"  Operation: new = old ([+*]) (\d+|old)\s+",
        r"  Test: divisible by (\d+)\s+",
        r"    If true: throw to monkey (\d+)\s+",
        r"    If false: throw to monkey (\d+)"
    ))
    .unwrap()
});

struct Monkey {
    items: Vec<u64>,
    monkey_rule: MonkeyRule,
    inspect_count: u64,
}

struct MonkeyRule {
    operation: Operation,
    divisible_by: u64,
    target_true: u64,
    target_false: u64,
}

struct Operation {
    operation: OperationType,
    value: Operand,
}

impl Operation {
    fn inspect(&self, item: u64) -> u64 {
        let operand = match self.value {
            Operand::OLD => item,
            Operand::VALUE(x) => x,
        };
        match self.operation {
            OperationType::ADD => item + operand,
            OperationType::MUL => item * operand,
        }
    }
}

enum Operand {
    OLD,
    VALUE(u64),
}

enum OperationType {
    ADD,
    MUL,
}

fn main() -> Result<()> {
    let mut monkeys = parse()?;

    let mut lcm = 1;
    for monkey in monkeys.iter() {
        lcm *= monkey.monkey_rule.divisible_by;
    }

    for _ in 0..10000 {
        for i in 0..monkeys.len() {
            while !monkeys[i].items.is_empty() {
                let monkey = &mut monkeys[i];
                let mut item = monkey.items.remove(0);
                item = monkey.monkey_rule.operation.inspect(item);
                monkey.inspect_count += 1;
                //part 1
                // item /= 3;
                //part 2
                item %= lcm;
                let target_i = match item % monkey.monkey_rule.divisible_by {
                    0 => monkey.monkey_rule.target_true,
                    _ => monkey.monkey_rule.target_false,
                };
                monkeys[target_i as usize].items.push(item)
            }
        }
    }

    monkeys.sort_by_key(|m| m.inspect_count);
    monkeys.reverse();

    println!("{}", monkeys[0].inspect_count * monkeys[1].inspect_count);

    Ok(())
}

fn parse() -> Result<Vec<Monkey>> {
    let mut buf: String = Default::default();
    io::stdin().read_to_string(&mut buf)?;

    Ok(RE
        .captures_iter(&buf)
        .map(|capture| {
            println!("capture {}", capture.get(0).unwrap().as_str());
            Ok(Monkey {
                items: capture
                    .get(1)
                    .context("capture group 1")?
                    .as_str()
                    .split(", ")
                    .map(|s| Ok(s.parse()?))
                    .collect::<Result<Vec<u64>>>()?,
                monkey_rule: MonkeyRule {
                    operation: Operation {
                        operation: match capture.get(2).context("capture group 2")?.as_str() {
                            "+" => OperationType::ADD,
                            "*" => OperationType::MUL,
                            s => bail!("unknown operand {s}"),
                        },
                        value: match capture.get(3).context("capture group 3")?.as_str() {
                            "old" => Operand::OLD,
                            s => Operand::VALUE(s.parse()?),
                        },
                    },
                    divisible_by: capture
                        .get(4)
                        .context("capture group 4")?
                        .as_str()
                        .parse()?,
                    target_true: capture
                        .get(5)
                        .context("capture group 5")?
                        .as_str()
                        .parse()?,
                    target_false: capture
                        .get(6)
                        .context("capture group 6")?
                        .as_str()
                        .parse()?,
                },
                inspect_count: 0,
            })
        })
        .collect::<Result<Vec<_>>>()?)
}
