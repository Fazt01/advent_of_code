use anyhow::{Context, Result};
use grid::{Coord, Grid, DIRECTIONS_CARDINAL};
use std::collections::HashMap;
use std::io::stdin;

type Tile = Option<u64>;

fn main() -> Result<()> {
    let grid = parse_input()?;

    // part 1
    let cost_to_end = shortest_path_to_end(&grid, 1024);
    println!("{}", cost_to_end.context("no path to end")?);

    // part 2
    let times = Vec::from_iter(1024..3450);
    let i = times.partition_point(|&time| {
        println!("{time}");
        shortest_path_to_end(&grid, time).is_some()
    });
    let time = times[i];
    let pos = grid
        .iter()
        .find(|(_, &t)| matches!(t, Some(x) if x == time))
        .context(format!("no wall at time {} found", time))?
        .0;
    println!("time {time}: position: {},{}", pos.x, pos.y);

    Ok(())
}

fn shortest_path_to_end(grid: &Grid<Tile>, time: u64) -> Option<u64> {
    let mut unvisited: HashMap<Coord, u64> = Default::default();

    for (coord, _) in grid.iter() {
        unvisited.insert(coord, u64::MAX);
    }

    unvisited.insert(Coord { x: 0, y: 0 }, 0);
    let end = Coord {
        x: grid.columns() as i64 - 1,
        y: grid.rows() as i64 - 1,
    };

    let mut cost_to_end = None;
    while !unvisited.is_empty() {
        let mut min_coord = Coord::default();
        let mut min = u64::MAX;
        for (&coord, &cost) in &unvisited {
            if cost < min {
                min = cost;
                min_coord = coord;
            }
        }

        if unvisited.remove(&min_coord).is_none() {
            return None;
        };
        if matches!(grid[min_coord], Some(wall_at_time) if wall_at_time <= time) {
            continue;
        }
        if min_coord == end {
            cost_to_end = Some(min);
            break;
        }

        for offset in DIRECTIONS_CARDINAL {
            let next_coord = min_coord + offset;
            if grid.is_valid(next_coord) {
                let new_cost = min + 1;
                unvisited.get_mut(&next_coord).map(|cost_so_far| {
                    if new_cost < *cost_so_far {
                        *cost_so_far = new_cost;
                    }
                });
            }
        }
    }

    cost_to_end
}

fn parse_input() -> Result<Grid<Tile>> {
    let mut grid: Grid<Tile> = Grid::new(71, 71);
    for (i, line) in stdin().lines().enumerate() {
        let line = line?;
        let coord = line
            .split(',')
            .map(|x| Ok(x.parse::<usize>()?))
            .collect::<Result<Vec<_>>>()?;
        let tile = grid.index_mut(coord[0], coord[1]);
        if tile.is_none() {
            *tile = Some(i as u64)
        }
    }
    Ok(grid)
}
