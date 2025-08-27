use anyhow::{Context, Result};
use lib::grid::Coord;
use itertools::Itertools;
use once_cell::sync::Lazy;
use regex::Regex;
use std::io::stdin;

#[derive(Debug)]
struct Machine {
    button_a: Coord,
    button_b: Coord,
    prize: Coord,
}

#[derive(Debug)]
struct WinCombination {
    button_a: i64,
    button_b: i64,
}

static RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?:Button A|Button B|Prize): X[+=](\d+), Y[+=](\d+)").unwrap());

fn main() -> Result<()> {
    let mut input = parse_input()?;

    for m in &mut input {
        part2_conversion(m);
    }

    println!(
        "{}",
        input
            .iter()
            .map(|m| win_combinations(m)
                .iter()
                .map(combination_price)
                .min()
                .unwrap_or_default())
            .sum::<u64>()
    );

    Ok(())
}

fn win_combinations(machine: &Machine) -> Vec<WinCombination> {
    // p = prize, a,b = buttons, new = the new coordinate after change of basis (new basis are a and b, transformed point is p)
    // p.x = new.x * a.x + new.y * b.x
    // p.y = new.x * a.y + new.y * b.y  // * (-a.x / a.y)
    //
    // (-a.x / a.y) * p.y = new.x * (-a.x) + new.y * b.y * (-a.x / a.y) // +
    //
    // p.x - (a.x / a.y) * p.y = new.y * b.x + new.y * b.y * (-a.x / a.y)
    // p.x - (a.x / a.y) * p.y = new.y * (b.x + b.y * (-a.x / a.y))
    // new.y = (p.x - (a.x / a.y) * p.y) / (b.x - b.y * (a.x / a.y))

    // new.x = (p.x - new.y * b.x) / a.x

    let a_x_div_y = machine.button_a.x as f64
        / machine.button_a.y as f64;
    let new_y = (machine.prize.x as f64 - a_x_div_y * machine.prize.y as f64)
        / (machine.button_b.x as f64 - machine.button_b.y as f64 * a_x_div_y);
    let new_x = (machine.prize.x as f64 - new_y * machine.button_b.x as f64)
        / machine.button_a.x as f64;

    let (round_x, round_y) = (new_x.round(), new_y.round());
    let hit = machine.prize.x == round_x as i64 * machine.button_a.x + round_y as i64 * machine.button_b.x
        && machine.prize.y == round_x as i64 * machine.button_a.y + round_y as i64 * machine.button_b.y;
    if hit {
        vec![WinCombination {
            button_a: round_x as i64,
            button_b: round_y as i64,
        }]
    } else {
        vec![]
    }
}

fn combination_price(combination: &WinCombination) -> u64 {
    3 * combination.button_a as u64 + combination.button_b as u64
}

fn part2_conversion(machine: &mut Machine) {
    const CONV: i64 = 10000000000000;
    machine.prize.x += CONV;
    machine.prize.y += CONV;
}

fn parse_input() -> Result<Vec<Machine>> {
    stdin()
        .lines()
        .chunks(4)
        .into_iter()
        .map(|mut chunk| -> Result<Machine> {
            Ok(Machine {
                button_a: parse_to_coord(chunk.next().context("expected button A line")??)?,
                button_b: parse_to_coord(chunk.next().context("expected button B line")??)?,
                prize: parse_to_coord(chunk.next().context("expected prize line")??)?,
            })
        })
        .collect()
}

fn parse_to_coord(line: String) -> Result<Coord> {
    let cap = RE
        .captures(line.as_str())
        .context("X and Y coordinates found")?;
    Ok(Coord {
        x: cap[1].parse()?,
        y: cap[2].parse()?,
    })
}
