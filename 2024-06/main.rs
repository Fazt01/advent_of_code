use std::collections::{HashSet};
use anyhow::{Context, Result};
use std::io::stdin;
use std::iter::repeat;
use owned_chars::{OwnedCharsExt};
use grid::{Coord, Grid, Offset, OFFSET_DOWN, OFFSET_RIGHT, OFFSET_UP, OFFSET_LEFT};

#[derive(Copy, Clone, Eq, Hash, PartialEq, Ord, PartialOrd)]
struct Guard {
    position: Coord,
    direction: Offset,
}

struct Puzzle {
    guard_start: Guard,
    grid: Grid<char>,
}

fn main() -> Result<()> {
    let mut puzzle = parse_input()?;

    let visited = part1(&puzzle);

    println!("{}", visited.len());

    let loopy_obstacles = part2(&mut puzzle, &visited)?;

    println!("{}", loopy_obstacles);

    Ok(())
}

fn part2(puzzle: &mut Puzzle, visited: &HashSet<Coord>) -> Result<i32> {
    let mut loopy_obstacles = 0;

    for &candidate in visited {
        let added_obstacle = &mut puzzle.grid[candidate];
        if *added_obstacle == '.' {
            *added_obstacle = '#';
        }

        if is_loop(&puzzle)? {
            loopy_obstacles += 1;
        }

        puzzle.grid[candidate] = '.';
    }



    Ok(loopy_obstacles)
}

fn is_loop(puzzle: &Puzzle) -> Result<bool> {
    let mut visited_states: Grid<i8> = Grid::from_lines_iter(repeat(repeat(0).take(puzzle.grid.columns())).take(puzzle.grid.rows()))?;
    let mut guard = puzzle.guard_start;
    'outer: loop {
        for (coord, point) in puzzle.grid.iter_line(guard.position, guard.direction) {
            if *point == '#' {
                guard.position = coord - guard.direction;
                guard.direction = guard.direction.rotate_right();
                continue 'outer
            }
            let bitmask = offset_to_bit(guard.direction);
            let ok = (visited_states[coord] & bitmask) != 0;
            visited_states[coord] |= bitmask;
            if ok {
                return Ok(true)
            };
        }
        return Ok(false)
    }
}

fn part1(puzzle: &Puzzle) -> HashSet<Coord> {
    let mut visited = HashSet::<Coord>::default();
    let mut guard = puzzle.guard_start;
    'outer: loop {
        for (coord, point) in puzzle.grid.iter_line(guard.position, guard.direction) {
            if *point == '#' {
                guard.position = coord - guard.direction;
                guard.direction = guard.direction.rotate_right();
                continue 'outer
            }
            visited.insert(coord);
        }
        break;
    }
    visited
}

fn offset_to_bit(offset: Offset) -> i8 {
    match offset {
        OFFSET_UP => 1,
        OFFSET_DOWN => 2,
        OFFSET_LEFT => 4,
        OFFSET_RIGHT => 8,
        _ => unreachable!(),
    }
}

fn parse_input() -> Result<Puzzle> {
    let grid = itertools::process_results(
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
    )??;

    let guard_start = grid.iter()
        .filter(|(_, &p)| p == '^')
        .map(|(c, _)| Guard{
            position: c,
            direction: OFFSET_UP,
        })
        .next()
        .context("position of guard not found")?;

    Ok(Puzzle{
        guard_start,
        grid,
    })
}