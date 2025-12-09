use std::collections::{HashMap};
use anyhow::{bail, Result};
use owned_chars::OwnedCharsExt;
use lib::grid::{Coord, Grid};
use std::io::stdin;

enum Point {
    Empty,
    Start,
    Splitter,
}

fn main() -> Result<()> {
    let grid = parse_input()?;

    let (split_count, end_rays) = propagate_rays(&grid);

    println!("{}", split_count);
    println!("{}", end_rays.iter().map(|(_, v)| v).sum::<i64>());

    Ok(())
}

fn propagate_rays(grid: &Grid<Point>) -> (i64, HashMap<usize, i64>) {
    let mut result = HashMap::default();
    let mut split_count = 0;

    for y in 0..grid.rows() {
        for x in 0..grid.columns() {
            let coord = Coord{x: x as i64, y: y as i64};
            match grid[coord] {
                Point::Empty => {}
                Point::Start => {result.insert(x, 1);}
                Point::Splitter => {
                    if let Some(removed) = result.remove(&x) {
                        split_count += 1;
                        if x > 0 {
                            *result.entry(x-1).or_default() += removed;
                        }
                        if x < grid.columns()-1 {
                            *result.entry(x+1).or_default() += removed;
                        }
                    }
                }
            }
        }
    }

    (split_count, result)
}

fn parse_input() -> Result<Grid<Point>> {
    Ok(Grid::from_lines_try_iter_map(
        stdin().lines().map(|s| s.map(|s| s.into_chars())),
        |_, char| {
            Ok(match char {
                '.' => Point::Empty,
                'S' => Point::Start,
                '^' => Point::Splitter,
                x => bail!("expected grid point, got '{}'", x),
            })
        },
    )?)
}
