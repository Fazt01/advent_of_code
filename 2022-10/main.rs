use std::{io};
use anyhow::{Result, Ok, bail};

struct Program {
    instructions: Vec<Instruction>,
}

impl Program {
    fn run(&self) -> ProgramRun<'_> {
        ProgramRun {
            program: &self,
            cycle: 0,
            instruction_pointer: 0,
            reg_x: 1,
            remaining_to_execute: 0,
        }
    }
}

struct ProgramRun<'a> {
    program: &'a Program,
    cycle: u64,
    instruction_pointer: u64,
    reg_x: i64,
    remaining_to_execute: u8,
}

impl<'a> Iterator for ProgramRun<'a> {
    type Item = (u64, i64);

    fn next(&mut self) -> Option<Self::Item> {
        self.cycle += 1;
        if self.remaining_to_execute > 0 {
            self.remaining_to_execute -= 1;
            return match self.program.instructions[self.instruction_pointer as usize - 1] {
                Instruction::AddX(v) => {
                    let old_reg_x = self.reg_x;
                    self.reg_x += v;
                    Some((self.cycle, old_reg_x))
                }
                _ => unreachable!(),
            };
        }
        self.instruction_pointer += 1;
        if self.instruction_pointer - 1 >= self.program.instructions.len() as u64 {
            return None;
        }
        match self.program.instructions[self.instruction_pointer as usize - 1] {
            Instruction::Noop => {}
            Instruction::AddX(_) => {
                self.remaining_to_execute = 1;
            }
        }
        Some((self.cycle, self.reg_x))
    }
}

enum Instruction {
    AddX(i64),
    Noop,
}


fn main() -> Result<()> {
    let program = parse()?;

    let mut sum = 0;
    for (cycle, reg_x) in program.run() {
        if cycle == 20 || (cycle > 20 && (cycle - 20) % 40 == 0) {
            sum += cycle as i64 * reg_x
        }

        draw(cycle, reg_x);

        if cycle % 40 == 0 {
            println!();
        }
    }

    println!("{}", sum);

    Ok(())
}

fn draw(cycle: u64, reg_x: i64) {
    let cycle = cycle % 40;
    if cycle as i64 >= reg_x && cycle as i64 <= reg_x + 2 {
        print!("#")
    } else {
        print!(".")
    }
}

fn parse() -> Result<Program> {
    Ok(
        Program {
            instructions: io::stdin()
                .lines()
                .map(|line| {
                    let line = line?;
                    let split = line.split_whitespace().collect::<Vec<&str>>();
                    Ok(match split.as_slice() {
                        ["noop"] => Instruction::Noop,
                        ["addx", v] => Instruction::AddX(v.parse::<i64>()?),
                        _ => bail!("invalid operation")
                    })
                })
                .collect::<Result<_>>()?
        }
    )
}