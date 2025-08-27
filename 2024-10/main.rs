use anyhow::{bail, Result};
use lib::grid::{Coord, Grid, DIRECTIONS_CARDINAL};
use std::collections::HashSet;
use std::io::stdin;

fn main() -> Result<()> {
    let grid = parse_input()?;

    let mut part1 = 0;
    let mut part2 = 0;
    for (coord, _) in grid.iter() {
        part1 += count_trail_ends_from(&grid, coord, 0).len();
        part2 += count_trails_from(&grid, coord, 0);
    }
    println!("{part1}");
    println!("{part2}");

    Ok(())
}

fn count_trails_from(grid: &Grid<u8>, coord: Coord, start_height: u8) -> u64 {
    if grid[coord] != start_height {
        return 0;
    }
    if grid[coord] == 9 {
        return 1;
    }
    let mut sum = 0;
    for offset in DIRECTIONS_CARDINAL {
        if grid.is_valid(coord + offset) {
            sum += count_trails_from(&grid, coord + offset, start_height + 1)
        }
    }
    sum
}

fn count_trail_ends_from(grid: &Grid<u8>, coord: Coord, start_height: u8) -> HashSet<Coord> {
    if grid[coord] != start_height {
        return HashSet::default();
    }
    if grid[coord] == 9 {
        return HashSet::from([coord]);
    }
    let mut ends = HashSet::<Coord>::default();
    for offset in DIRECTIONS_CARDINAL {
        if grid.is_valid(coord + offset) {
            ends = ends.union(&count_trail_ends_from(&grid, coord + offset, start_height + 1)).cloned().collect()
        }
    }
    ends
}

fn parse_input() -> Result<Grid<u8>> {
    let grid = Grid::from_lines_try_iter(
        stdin().lines().map(
            |line| -> Result<_> {
                Ok(line.map(|line| {
                    line.into_bytes().into_iter().map(|c| {
                        if c.is_ascii_digit() {
                            Ok(c - b'0')
                        } else {
                            bail!("non digit character")
                        }
                    })
                })?)
            }
        )
    )?;

    Ok(grid)
}