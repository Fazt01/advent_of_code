use std::cmp::{max, min};
use std::collections::BTreeSet;
use std::io;
use std::ops::Bound;
use anyhow::{Result, Ok};

struct Space {
    galaxies: Vec<Galaxy>,
    empty_rows: BTreeSet<u64>,
    empty_columns: BTreeSet<u64>,
}

impl Space {
    fn extended_distance(&self, galaxy1: &Galaxy, galaxy2: &Galaxy, expansion_multiplier: u64) -> u64 {
        let regular_distance = galaxy1.x.abs_diff(galaxy2.x) + galaxy1.y.abs_diff(galaxy2.y);
        let extension = count(&self.empty_rows, galaxy1.y, galaxy2.y)
            + count(&self.empty_columns, galaxy1.x, galaxy2.x);
        regular_distance + extension * (expansion_multiplier-1) // -1 for the distance already accounted by regular distance
    }
}

fn count(set: &BTreeSet<u64>, x: u64, y: u64) -> u64 {
    if x == y {
        return 0;
    }
    set.range((
        Bound::Excluded(min(x, y)),
        Bound::Excluded(max(x, y)),
    )).count() as u64
}

struct Galaxy {
    x: u64,
    y: u64,
}

fn main() -> Result<()> {
    let space = parse()?;

    let mut sum: u64 = 0;
    let mut sum2: u64 = 0;
    for i in 0..space.galaxies.len() {
        for j in i..space.galaxies.len() {
            sum += space.extended_distance(&space.galaxies[i], &space.galaxies[j], 2);
            sum2 += space.extended_distance(&space.galaxies[i], &space.galaxies[j], 1_000_000);
        }
    }

    println!("{}", sum);
    println!("{}", sum2);

    Ok(())
}

fn parse() -> Result<Space> {
    let stdin = io::stdin();
    let mut rows: Option<u64> = None;
    let mut columns: Option<u64> = None;
    let mut occupied_rows = BTreeSet::<u64>::new();
    let mut occupied_columns = BTreeSet::<u64>::new();
    let mut galaxies = Vec::<Galaxy>::new();
    for (y, line) in stdin.lines().enumerate() {
        let line = line?;
        if columns.is_none() {
            columns = Some(line.chars().count() as u64)
        }
        rows = Some(rows.unwrap_or(0) + 1);
        for (x, c) in line.chars().enumerate() {
            match c {
                '#' => {
                    occupied_rows.insert(y as u64);
                    occupied_columns.insert(x as u64);
                    galaxies.push(Galaxy {
                        x: x as u64,
                        y: y as u64,
                    })
                }
                _ => {}
            };
        }
    }

    Ok(Space {
        galaxies,
        empty_rows: BTreeSet::from_iter(0..rows.unwrap()).difference(&occupied_rows).map(|x| *x).collect(),
        empty_columns: BTreeSet::from_iter(0..columns.unwrap()).difference(&occupied_columns).map(|x| *x).collect(),
    })
}

