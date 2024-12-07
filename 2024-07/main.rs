use anyhow::{bail, Result};
use std::io::stdin;


#[derive(Default)]
struct Puzzle {
    equations: Vec<Equation>
}

#[derive(Default)]
struct Equation {
    result: i64,
    operands: Vec<i64>
}

fn main() -> Result<()> {
    let puzzle = parse_input()?;

    let result = solve(&puzzle);

    println!("{result}");

    Ok(())
}

fn solve(puzzle: &Puzzle) -> i64 {
    let mut sum = 0;
    for equation in &puzzle.equations {
        if result_reachable(equation.result, equation.operands[0], &equation.operands[1..]) {
            sum += equation.result
        }
    }
    sum
}

fn result_reachable(result: i64, accumulator: i64, operands: &[i64]) -> bool {
    if operands.is_empty() {
        return result == accumulator
    }
    result_reachable(result, accumulator + operands[0], &operands[1..])
        || result_reachable(result, accumulator * operands[0], &operands[1..])
        // part2
        || result_reachable(result, concatenate(accumulator, operands[0]), &operands[1..])
}

fn concatenate(lhs: i64, rhs: i64) -> i64 {
    lhs * 10_i64.pow(rhs.to_string().len() as u32) + rhs
}

fn parse_input() -> Result<Puzzle> {
    let mut result = Puzzle::default();

    for line in stdin().lines() {
        let line = line?;
        let split: Vec<&str> = line.split(": ").collect();
        if split.len() != 2 {
            bail!("not found exactly one ': '")
        }
        let operands = split[1]
            .split(' ')
            .map(|x| -> Result<_> {
                Ok(x.parse::<i64>()?)
            });
        result.equations.push(Equation{
            result: split[0].parse()?,
            operands: operands.collect::<Result<_>>()?,
        })
    }

    Ok(result)
}