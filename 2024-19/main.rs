use anyhow::{Context, Result};
use itertools::Itertools;
use owned_chars::OwnedCharsExt;
use std::cmp::min;
use std::collections::HashMap;
use std::io::stdin;

type Pattern = Vec<char>;

struct Puzzle {
    available: Vec<Pattern>,
    target: Vec<Pattern>,
}

fn main() -> Result<()> {
    let input = parse_input()?;

    let count = input
        .target
        .iter()
        .filter(|x| is_possible(*x, &input.available))
        .count();

    println!("{count}");

    let mut cache: HashMap<Pattern, u64> = Default::default();
    let count = input
        .target
        .iter()
        .map(|x| possible_ways(&x, &input.available, &mut cache))
        .sum::<u64>();

    println!("{count}");

    Ok(())
}

fn is_possible(target: &[char], available: &Vec<Pattern>) -> bool {
    if target.is_empty() {
        return true;
    }
    available.iter().any(|x| {
        target.starts_with(x) && is_possible(&target[min(target.len(), x.len())..], available)
    })
}

fn possible_ways(
    target: &[char],
    available: &Vec<Pattern>,
    cache: &mut HashMap<Pattern, u64>,
) -> u64 {
    if let Some(&cached) = cache.get(target) {
        return cached;
    }
    if target.is_empty() {
        return 1;
    }
    let ways = available
        .iter()
        .map(|x| {
            if target.starts_with(x) {
                possible_ways(&target[min(target.len(), x.len())..], available, cache)
            } else {
                0
            }
        })
        .sum();

    cache.insert(target.into(), ways);
    ways
}

fn parse_input() -> Result<Puzzle> {
    let mut lines = stdin().lines();
    let available = lines
        .next()
        .context("expected line of available patterns")??
        .split(", ")
        .map(|s| s.chars().collect_vec())
        .collect_vec();
    lines.next();
    let target = lines
        .map(|x| Ok(x?.into_chars().collect_vec()))
        .collect::<Result<Vec<_>>>()?;

    Ok(Puzzle { available, target })
}
