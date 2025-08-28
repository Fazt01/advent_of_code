use anyhow::{Context, Ok, Result};
use lib::grid::{Coord, Offset};
use std::collections::HashMap;
use std::io::stdin;

struct Span {
    from: Coord,
    to: Coord,
}

enum Material {
    Air,
    Rock,
    Sand,
}

fn main() -> Result<()> {
    let spans = parse()?;

    let mut solids = spans_to_solids(&spans);

    let lowest = solids_to_bounds(&solids).context("empty scan")?.max_y;

    let spawn = Coord { x: 500, y: 0 };
    let mut part1done = false;

    let mut i = 0;
    loop {
        let rest_at = drop_till_rest(&solids, spawn, lowest+2);
        if rest_at.y > lowest && !part1done {
            part1done = true;
            println!("{i}");
        }
        solids.insert(rest_at, Material::Sand);
        i += 1;
        if rest_at == spawn {
            break
        }
    }

    println!("{i}");

    Ok(())
}

static OFFSETS_ATTEMPTS: &[Offset] = &[
    Offset { x: 0, y: 1 },
    Offset { x: -1, y: 1 },
    Offset { x: 1, y: 1 },
];

fn drop_till_rest(solids: &HashMap<Coord, Material>, start: Coord, lowest: i64) -> Coord {
    let mut current = start;
    'outer: loop {
        for &offset in OFFSETS_ATTEMPTS {
            let moved = current + offset;
            if !matches!(solids.get(&moved).unwrap_or(&Material::Air), Material::Air) {
                continue;
            }
            if moved.y >= lowest {
                return current
            }
            current = moved;
            continue 'outer;
        }
        return current;
    }
}

fn spans_to_solids(spans: &Vec<Span>) -> HashMap<Coord, Material> {
    let mut result = HashMap::new();
    for span in spans {
        let mut current = span.from;
        let dif = span.to - span.from;
        let direction = Offset {
            x: sign(dif.x),
            y: sign(dif.y),
        };
        result.insert(current, Material::Rock);
        while current != span.to {
            current = current + direction;
            result.insert(current, Material::Rock);
        }
    }

    result
}

struct Bounds {
    min_x: i64,
    max_x: i64,
    min_y: i64,
    max_y: i64,
}

fn solids_to_bounds(map: &HashMap<Coord, Material>) -> Option<Bounds> {
    let first = map.keys().next()?;
    let mut bounds = Bounds {
        min_x: first.x,
        max_x: first.x,
        min_y: first.y,
        max_y: first.y,
    };
    for c in map.keys() {
        if c.x < bounds.min_x {
            bounds.min_x = c.x
        }
        if c.x > bounds.max_x {
            bounds.max_x = c.x
        }
        if c.y < bounds.min_y {
            bounds.min_y = c.y
        }
        if c.y > bounds.max_y {
            bounds.max_y = c.y
        }
    }
    Some(bounds)
}

fn print_map(map: &HashMap<Coord, Material>) -> () {
    if let Some(bounds) = solids_to_bounds(map) {
        for y in bounds.min_y..=bounds.max_y {
            for x in bounds.min_x..=bounds.max_x {
                let c = match map.get(&Coord { x, y }).unwrap_or(&Material::Air) {
                    Material::Air => '.',
                    Material::Rock => '#',
                    Material::Sand => 'o',
                };
                print!("{c}")
            }
            println!();
        }
    }
}

fn sign(v: i64) -> i64 {
    if v > 0 {
        1
    } else if v < 0 {
        -1
    } else {
        0
    }
}

fn parse() -> Result<Vec<Span>> {
    stdin()
        .lines()
        .map(|line| {
            let line = line?;
            Ok(line
                .split(" -> ")
                .map(|s| {
                    let mut splits = s.split(',');
                    Ok(Coord {
                        x: splits
                            .next()
                            .context("no first coordinate")?
                            .parse::<i64>()?,
                        y: splits
                            .next()
                            .context("no second coordinate")?
                            .parse::<i64>()?,
                    })
                })
                .collect::<Result<Vec<_>>>()?
                .windows(2)
                .map(|window| Span {
                    from: window[0],
                    to: window[1],
                })
                .collect::<Vec<_>>())
        })
        .try_fold::<_, _, Result<Vec<Span>>>(vec![], |mut acc, item| {
            acc.append(&mut item?);
            Ok(acc)
        })
}
