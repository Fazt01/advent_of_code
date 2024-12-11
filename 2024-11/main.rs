use anyhow::Result;
use std::collections::HashMap;
use std::io::{stdin, Read};

type StoneEngraving = u64;

type CompactStoneLine = HashMap<StoneEngraving, u64>;

fn main() -> Result<()> {
    let input = parse_input()?;
    let mut input_line = vec_to_stone_line(&input);

    for i in 0..75 {
        blink_line(&mut input_line);
        println!("blink {i} results in {} stones", input_line.values().sum::<u64>());
    }

    println!("{}", input_line.values().sum::<u64>());

    Ok(())
}

fn vec_to_stone_line(stones: &Vec<StoneEngraving>) -> CompactStoneLine {
    let mut result = CompactStoneLine::new();
    for stone in stones {
        *result.entry(*stone).or_default() += 1;
    }
    result
}

fn blink(input: &mut Vec<StoneEngraving>) {
    let mut next = Vec::new();

    for &stone in &*input {
        match stone {
            0 => {
                next.push(1);
            }
            x if number_of_digits(x) % 2 == 0 => {
                let (a, b) = split_stone(x);
                next.push(a);
                next.push(b);
            }
            x => {
                next.push(x * 2024);
            }
        }
    }

    *input = next;
}

fn blink_line(input: &mut CompactStoneLine) {
    let mut next = CompactStoneLine::new();

    for (&stone, count) in &*input {
        match stone {
            0 => {
                *next.entry(1).or_default() += count;
            }
            x if number_of_digits(x) % 2 == 0 => {
                let (a, b) = split_stone(x);
                *next.entry(a).or_default() += count;
                *next.entry(b).or_default() += count;
            }
            x => {
                *next.entry(x * 2024).or_default() += count;
            }
        }
    }

    *input = next;
}

fn number_of_digits(n: StoneEngraving) -> usize {
    n.to_string().len()
}

fn split_stone(n: StoneEngraving) -> (StoneEngraving, StoneEngraving) {
    let s = n.to_string();
    let new_stone_digits = s.len() / 2;
    (
        n / (10 as StoneEngraving).pow(new_stone_digits as u32),
        n % (10 as StoneEngraving).pow(new_stone_digits as u32),
    )
}

fn parse_input() -> Result<Vec<StoneEngraving>> {
    let mut buf = Default::default();

    stdin().read_to_string(&mut buf)?;

    Ok(buf
        .split_ascii_whitespace()
        .map(|x| Ok(x.parse()?))
        .collect::<Result<Vec<_>>>()?)
}
