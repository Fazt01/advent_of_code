use anyhow::{bail, Context, Result};
use grid::{Coord, Grid, Offset, DIRECTIONS_CARDINAL};
use owned_chars::OwnedCharsExt;
use std::collections::HashMap;
use std::io::stdin;

struct Puzzle {
    grid: Grid<char>,
    start: Coord,
    end: Coord,
}

fn main() -> Result<()> {
    let input = parse_input()?;

    let mut distances: HashMap<Coord, u64> = Default::default();

    let mut distance = 0;
    let mut pos = input.start;
    'outer: loop {
        distances.insert(pos, distance);
        for dir in DIRECTIONS_CARDINAL {
            let next = pos + dir;

            if input.grid[next] == '.' && distances.get(&next).is_none() {
                distance += 1;
                pos = next;
                continue 'outer;
            }
        }
        break;
    }

    let offsets = offsets_within_manhattan_distance(20);

    let count = distances
        .iter()
        .map(|(&pos, &dist)| {
            offsets
                .iter()
                .map({
                    let distances = &distances;
                    move |&offset| {
                        let next = pos + offset;
                        if let Some(&dist_from_next) = distances.get(&next) {
                            let cheat_dist = (offset.x.abs() + offset.y.abs()) as u64;
                            if dist_from_next > dist + cheat_dist {
                                return dist_from_next - dist - cheat_dist;
                            }
                        }
                        return 0;
                    }
                })
                .filter(|&x| x > 0)
        })
        .flatten()
        .filter(|&x| x >= 100)
        .count();

    println!("{count}");

    Ok(())
}

fn offsets_within_manhattan_distance(dist: u64) -> Vec<Offset> {
    let mut res: Vec<Offset> = Default::default();

    for y in -(dist as i64)..=dist as i64 {
        let abs_x_max = dist as i64 - y.abs();
        for x in -abs_x_max..=abs_x_max {
            res.push(Offset { x, y })
        }
    }

    res
}

fn parse_input() -> Result<Puzzle> {
    let mut start = None;
    let mut end = None;
    let grid = Grid::from_lines_try_iter_map(
        stdin()
            .lines()
            .map(|line| -> Result<_> { Ok(line?.into_chars()) }),
        |pos, c| {
            Ok(match c {
                'S' => {
                    if !start.is_none() {
                        bail!("duplicate start")
                    }
                    start = Some(pos);
                    '.'
                }
                'E' => {
                    if !end.is_none() {
                        bail!("duplicate start")
                    }
                    end = Some(pos);
                    '.'
                }
                x => x,
            })
        },
    )?;

    Ok(Puzzle {
        grid,
        start: start.context("no start found")?,
        end: end.context("no end found")?,
    })
}
