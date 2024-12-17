use std::collections::HashSet;
use anyhow::{bail, Context, Result};
use std::io::{stdin, BufRead};
use itertools::Itertools;

#[derive(Copy, Clone, Debug)]
struct Registers {
    a: u64,
    b: u64,
    c: u64,
}

struct Puzzle {
    regs: Registers,
    program: Vec<u8>,
}

#[derive(Debug)]
struct State {
    instruction_ptr: usize,
    regs: Registers,
    output: Vec<u8>,
}


fn main() -> Result<()> {
    let input = parse_input()?;

    // part 1
    let state = State{
        instruction_ptr: 0,
        regs: input.regs,
        output: Default::default(),
    };

    let output = run(state, &input.program)?;

    println!("{}", output.iter().map(|x| x.to_string()).join(","));

    // part 2
    let mut candidates: HashSet<u64> = [0].into();
    for output_suffix_len in 1..=input.program.len() {
        println!("suffix len {output_suffix_len}");
        let suffix_start = input.program.len() - output_suffix_len;
        let mut new_candidates: HashSet<u64> = Default::default();
        for candidate in &candidates{
            let searched_output = &input.program[suffix_start..];
            // output of each digit always depends only on lower 10 bits of A, as:
            // B <- A % 8   // loads to B number 0 through 7
            // C <- A >> B  // can discard up to 7 A's lower bits
            // when outputting, only lower 3 bits of result are relevant
            for reg_a_lower in 0..1024 {
                let reg_a = candidate * 8 + reg_a_lower;
                let state = State{
                    instruction_ptr: 0,
                    regs: Registers{
                        a: reg_a,
                        b: input.regs.b,
                        c: input.regs.c,
                    },
                    output: Default::default(),
                };
                let output = run(state, &input.program)?;
                if searched_output.eq(&output[..]) {
                    if output_suffix_len == input.program.len() {
                        println!("{reg_a} {} match len {output_suffix_len}", output.iter().map(|x| x.to_string()).join(","));
                    }
                    new_candidates.insert(reg_a);
                }
            }
        }
        candidates = new_candidates;
    }

    println!("{}", candidates.iter().min().context("no remaining candidate for full suffix length")?);

    Ok(())
}

fn run(mut state: State, program: &Vec<u8>) -> Result<Vec<u8>>{
    while state.instruction_ptr < program.len() {
        // println!("{:?}", state);
        let mut next_instruction_ptr = state.instruction_ptr + 2;
        let instruction = program[state.instruction_ptr];
        let operand = program[state.instruction_ptr + 1];
        match instruction {
            0 => state.regs.a = state.regs.a / 2_u64.pow(resolve_operand(operand, &state)? as u32),
            1 => state.regs.b = state.regs.b ^ (operand as u64),
            2 => state.regs.b = resolve_operand(operand, &state)? % 8,
            3 => if state.regs.a != 0 {
                next_instruction_ptr = operand as usize;
            },
            4 => state.regs.b = state.regs.b ^ state.regs.c,
            5 => state.output.push((resolve_operand(operand, &state)? % 8) as u8),
            6 => state.regs.b = state.regs.a / 2_u64.pow(resolve_operand(operand, &state)? as u32),
            7 => state.regs.c = state.regs.a / 2_u64.pow(resolve_operand(operand, &state)? as u32),
            _ => bail!("invalid instruction code {instruction}")
        }
        state.instruction_ptr = next_instruction_ptr;
    }
    Ok(state.output)
}

fn resolve_operand(operand: u8, state: &State) -> Result<u64> {
    Ok(match operand {
        x if x <= 3 => x as u64,
        4 => state.regs.a,
        5 => state.regs.b,
        6 => state.regs.c,
        _ => bail!("unexpected operand value {}", operand)
    })
}

fn parse_input() -> Result<Puzzle> {
    let mut stdin = stdin().lock();
    let mut line = String::new();
    stdin.read_line(&mut line)?;
    let reg_a = line.strip_prefix("Register A: ").context("invalid register A input line")?.trim().parse()?;
    line = String::new();
    stdin.read_line(&mut line)?;
    let reg_b = line.strip_prefix("Register B: ").context("invalid register B input line")?.trim().parse()?;
    line = String::new();
    stdin.read_line(&mut line)?;
    let reg_c = line.strip_prefix("Register C: ").context("invalid register C input line")?.trim().parse()?;
    line = String::new();
    stdin.read_line(&mut line)?;
    line = String::new();
    stdin.read_line(&mut line)?;
    let program_str = line.strip_prefix("Program: ").context("invalid program input line")?.trim();
    let program = program_str.split(',').map(|c| Ok(c.parse::<u8>()?)).collect::<Result<Vec<_>>>()?;
    Ok(Puzzle{
        regs: Registers {
            a: reg_a,
            b: reg_b,
            c: reg_c,
        },
        program,
    })
}
