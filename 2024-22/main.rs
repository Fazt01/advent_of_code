use anyhow::Result;
use grid::{Coord, Grid};
use itertools::Itertools;
use std::collections::HashMap;
use std::io::stdin;

type Code = Vec<u8>;

struct Keyboard {
    grid: Grid<u8>,
    coords: HashMap<u8, Coord>,
}

impl From<Grid<u8>> for Keyboard {
    fn from(value: Grid<u8>) -> Self {
        let mut coords: HashMap<u8, Coord> = Default::default();

        for (coord, &button) in value.iter() {
            coords.insert(button, coord);
        }

        Keyboard {
            grid: value,
            coords,
        }
    }
}

const KEYBOARD_COUNT: u64 = 26;

fn main() -> Result<()> {
    let input = parse_input()?;

    // part 1
    let mut sum = 0;
    for &init in &input {
        sum += run(init, 2000);
    }
    println!("{sum}");

    // part 2
    let mut seq_prices = vec![];
    for &init in &input {
        seq_prices.push(run_sequence_output(init, 2000));
    }

    let mut max = 0;
    for (i, t) in itertools::iproduct!(-9..=9, -9..=9, -9..=9, -9..=9).enumerate() {
        if i % 10000 == 0 {
            println!("{i}, {t:?}")
        }
        let sequence = vec![t.0, t.1, t.2, t.3];
        let mut sum = 0;
        for input_prices in &seq_prices {
            sum += input_prices.get(&sequence).cloned().unwrap_or_default() as u64;
        }
        if max < sum {
            max = sum
        }
    }
    println!("{max}");

    Ok(())
}

const MOD: u64 = 16777216;

fn run(init: u64, iterations: u64) -> u64 {
    let mut v = init;
    for _ in 0..iterations {
        v ^= (v * 64) % MOD;
        v ^= (v / 32) % MOD;
        v ^= (v * 2048) % MOD;
    }
    v
}

type Sequence = Vec<i8>;

fn run_sequence_output(init: u64, iterations: u64) -> HashMap<Sequence, i8> {
    let mut v = init;
    let mut res: HashMap<Sequence, i8> = Default::default();
    let mut seq = vec![];
    for _ in 0..iterations {
        let prev_price = (v % 10) as i8;
        v ^= (v * 64) % MOD;
        v ^= (v / 32) % MOD;
        v ^= (v * 2048) % MOD;
        let price = (v % 10) as i8;
        let change = price - prev_price;
        if seq.len() == 4 {
            seq.remove(0);
        }
        seq.push(change);
        if seq.len() == 4 {
            res.entry(seq.clone()).or_insert(price);
        }
    }
    res
}

fn parse_input() -> Result<Vec<u64>> {
    Ok(stdin()
        .lines()
        .map(|line| -> Result<_> { Ok(line?.parse::<u64>()?) })
        .try_collect()?)
}
