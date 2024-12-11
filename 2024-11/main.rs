use anyhow::Result;
use std::io::{stdin, Read};

type StoneEngraving = u64;

fn main() -> Result<()> {
    let mut input = parse_input()?;

    for i in 0..25 {
        blink(&mut input);
        println!("blink {i} results in {} stones", input.len());
    }

    println!("{}", input.len());

    Ok(())
}

fn blink(input: &mut Vec<StoneEngraving>) {
    let mut next = Vec::default();

    for &stone in &*input {
        match stone {
            0 => {
                next.push(1)
            }
            x if number_of_digits(x) % 2 == 0 => {
                let (a, b) = split_stone(x);
                next.push(a);
                next.push(b);
            }
            x => {
                next.push(x * 2024)
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
    (n / (10 as StoneEngraving).pow(new_stone_digits as u32), n % (10 as StoneEngraving).pow(new_stone_digits as u32))
}

fn parse_input() -> Result<Vec<StoneEngraving>> {
    let mut buf = Default::default();

    stdin().read_to_string(&mut buf)?;

    Ok(buf
        .split_ascii_whitespace()
        .map(|x| Ok(x.parse()?))
        .collect::<Result<Vec<_>>>()?
    )
}
