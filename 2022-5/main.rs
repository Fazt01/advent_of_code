use std::io;
use std::ops::{Deref};
use anyhow::{Context, Error};
use once_cell::sync::Lazy;
use regex::{Regex};

static RE2: Lazy<Regex> = Lazy::new(|| Regex::new(r#"move (\d+) from (\d+) to (\d+)"#).unwrap());

fn main() -> Result<(), Error> {
    Lazy::get(&RE2);
    let stdin = io::stdin();
    let mut parse_state = 0;
    let mut stacks: Vec<Vec<String>> = Vec::new();
    for line in stdin.lines() {
        let line = line?;
        match parse_state {
            0 => {
                if line.starts_with(" 1") {
                    parse_state = 1;
                    stacks.iter_mut().for_each(|stack| { stack.reverse() });
                    continue;
                }
                let row = parse_stack_row(&line)?;
                stacks.resize_with(row.len(), Vec::new);
                for (i, item) in row.iter().enumerate() {
                    match item {
                        None => {}
                        Some(a) => stacks[i].push(a.to_string())
                    }
                }
            }
            1 => {
                parse_state = 2;
            }
            2 => {
                let mov = parse_move(&line)?;
                // first half
                // for _ in 0..(mov[0] as usize) {
                //     let popped = stacks[mov[1] as usize - 1usize].pop().context("stack empty")?;
                //     stacks[mov[2] as usize - 1usize].push(popped);
                // }

                // second half
                let amount = mov[0] as usize;
                let from_stack = &mut stacks[mov[1] as usize - 1usize];
                let copy = Vec::from_iter(from_stack.drain(from_stack.len() - amount..));
                stacks[mov[2] as usize - 1usize].extend_from_slice(copy.as_slice());
            }
            _ => {
                continue;
            }
        }
    }

    stacks.iter().try_for_each(|x| -> Result<(), Error> {
        print!("{}", x.last().map(String::deref).unwrap_or(" "));
        Ok(())
    })?;

    Ok(())
}

fn parse_stack_row(s: &str) -> Result<Vec<Option<String>>, Error> {
    let mut res: Vec<Option<String>> = Vec::default();
    let mut spaces = 0;
    for c in s.chars() {
        match c {
            '[' => {
                spaces = 0;
            }
            ']' => {}
            x if c >= 'A' && c <= 'z' => {
                res.push(Some(x.into()))
            }
            ' ' => {
                spaces += 1;
                if spaces >= 4 {
                    spaces = 0;
                    res.push(None)
                }
            }
            _ => {}
        }
    }
    Ok(res)
}

fn parse_move(s: &str) -> Result<[i64; 3], Error> {
    let captures = RE2.captures(s).context("no match")?;
    if captures.len() != 4 {
        return Err(Error::msg("not 3 captured groups"));
    }

    let f = |i| -> Result<i64, Error> {
        captures.get(i).context("no match")?.as_str().parse::<i64>().context("parse int")
    };

    return Ok([f(1)?, f(2)?, f(3)?]);
}