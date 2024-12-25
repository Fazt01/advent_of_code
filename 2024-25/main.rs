use anyhow::{Result, Ok};
use grid::{Coord, Grid, OFFSET_RIGHT, OFFSET_UP};
use itertools::Itertools;
use owned_chars::OwnedCharsExt;
use std::io::stdin;

fn main() -> Result<()> {
    let input = parse_input()?;

    let mut lock_heights: Vec<Vec<u64>> = vec![];
    let mut key_heights: Vec<Vec<u64>> = vec![];

    for item in &input {
        let (is_key, heights) = to_heights(item);
        if is_key {
            key_heights.push(heights);
        } else {
            lock_heights.push(heights);
        }
    }

    let mut sum = 0;
    for lock in &lock_heights {
        for key in &key_heights {
            if is_viable(lock, key) {
                sum += 1
            }
        }
    }

    println!("{sum}");

    Ok(())
}

fn is_key(grid: &Grid<char>) -> bool {
    grid.iter_line(Coord { x: 0, y: 0 }, OFFSET_RIGHT)
        .all(|(_, &x)| x == '.')
}

fn to_heights(grid: &Grid<char>) -> (bool, Vec<u64>) {
    let is_key = is_key(grid);
    (
        is_key,
        (0..grid.columns())
            .map(|col| {
                grid.iter_line(
                    Coord {
                        x: col as i64,
                        y: (grid.rows() - 1) as i64,
                    },
                    OFFSET_UP,
                )
                .filter(|(_, &x)| (x == '#') == is_key)
                .count() as u64
            })
            .collect_vec(),
    )
}

fn is_viable(lock: &Vec<u64>, key: &Vec<u64>) -> bool {
    (0..lock.len()).all(|i| lock[i] >= key[i])
}

fn parse_input() -> Result<Vec<Grid<char>>> {
    let mut lines = stdin().lines();
    let mut grids: Vec<Grid<char>> = vec![];
    loop {
        let grid_lines = lines
            .by_ref()
            .take_while(|line| matches!(line, Result::Ok(line) if !line.is_empty()));
        let grid = Grid::from_lines_try_iter(
            grid_lines.map(|line| Ok(line?.into_chars().map(|x| Ok(x)))),
        )?;
        if grid.columns() == 0 {
            break;
        }
        grids.push(grid);
    }

    Ok(grids)
}
