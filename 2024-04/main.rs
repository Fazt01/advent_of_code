use anyhow::{Result};
use std::io::stdin;
use owned_chars::{OwnedCharsExt};
use lib::grid::{Grid, DIRECTIONS_8, DIRECTIONS_X};

fn main() -> Result<()> {
    let grid = parse_input()?;

    println!("{}", part2(&grid));

    Ok(())
}

fn part1(grid: &Grid<char>) -> i32 {
    let mut count = 0;
    for (coord, point) in grid.iter() {
        const TARGET: &str = "XMAS";

        if *point != TARGET.chars().next().unwrap() {
            continue
        }
        for direction in DIRECTIONS_8 {
            if grid
                .iter_line(coord, direction)
                .map(|(_, &p)| p)
                .skip(1)
                .take(TARGET.len() - 1)
                .eq(TARGET[1..].chars()) {
                count += 1
            }
        }
    }
    count
}

fn part2(grid: &Grid<char>) -> i32 {
    let mut count = 0;
    for (coord, point) in grid.iter() {
        const TARGET: &str = "MAS";

        if *point != TARGET.chars().next().unwrap() {
            continue
        }
        for direction in DIRECTIONS_X {
            if grid
                .iter_line(coord, direction)
                .map(|(_, &p)| p)
                .skip(1)
                .take(TARGET.len() - 1)
                .eq(TARGET[1..].chars())
            {
                let mid_coord = coord + direction;
                let left_coord = mid_coord + direction.rotate_left();
                let right_coord = mid_coord + direction.rotate_right();
                if grid.is_valid(left_coord)
                    && grid.is_valid(right_coord)
                    && (
                    (grid[left_coord] == 'M' && grid[right_coord] == 'S')
                    // Checking for both orientation of the cross would actually double count the crosses.
                    // By assuming arbitrary but consistent direction, only one of the two will be counted.
                    // || (*grid.index_coord(left_coord) == 'S' && *grid.index_coord(right_coord) == 'M')
                )

                {
                    count += 1
                }
            }
        }
    }
    count
}

fn parse_input() -> Result<Grid<char>> {
    Ok(itertools::process_results(
        stdin().lines().map(
            |line| -> Result<_> {
                Ok(line.map(|line| {
                    line.into_chars()
                })?)
            }
        ),
        |line| {
            Grid::from_lines_iter(line)
        },
    )??)
}