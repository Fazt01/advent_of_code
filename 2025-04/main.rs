use anyhow::{bail, Result};
use lib::grid::{Grid, DIRECTIONS_8};
use std::io::stdin;

enum Point {
    Empty,
    Roll,
}

fn main() -> Result<()> {
    let mut grid = parse_input()?;

    let mut sum_part1 = 0;

    for (coord, point) in grid.iter() {
        if matches!(point, Point::Empty) {
            continue;
        }
        let mut neighbor_rolls = 0;
        for offset in DIRECTIONS_8 {
            let neighbor_coord = coord + offset;
            if grid.is_valid(neighbor_coord) && matches!(grid[neighbor_coord], Point::Roll) {
                neighbor_rolls += 1;
            }
        }
        if neighbor_rolls < 4 {
            sum_part1 += 1
        }
    }

    println!("{}", sum_part1);

    let mut sum_part2 = 0;

    let mut changed = true;
    while changed {
        changed = false;
        let mut to_remove = vec![];
        for (coord, point) in grid.iter() {
            if matches!(point, Point::Empty) {
                continue;
            }
            let mut neighbor_rolls = 0;
            for offset in DIRECTIONS_8 {
                let neighbor_coord = coord + offset;
                if grid.is_valid(neighbor_coord) && matches!(grid[neighbor_coord], Point::Roll) {
                    neighbor_rolls += 1;
                }
            }
            if neighbor_rolls < 4 {
                sum_part2 += 1;
                to_remove.push(coord)
            }
        }
        for coord in to_remove {
            grid[coord] = Point::Empty;
            changed = true
        }
    }

    println!("{}", sum_part2);

    Ok(())
}

fn parse_input() -> Result<Grid<Point>> {
    Ok(Grid::from_lines_try_iter(stdin().lines().map(|line| {
        line.map(|l| {
            l.as_bytes()
                .iter()
                .map(|byte| -> Result<Point> {
                    match byte {
                        b'.' => Ok(Point::Empty),
                        b'@' => Ok(Point::Roll),
                        x => bail!("expected . or @, got {}", x),
                    }
                })
                .collect::<Vec<_>>()
        })
    }))?)
}
