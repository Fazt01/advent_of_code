use anyhow::{Context, Result};
use lib::grid::{Coord, Grid, DIRECTIONS_CARDINAL};
use std::cmp::{Ordering, Reverse};
use std::collections::BinaryHeap;
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

#[derive(Eq, PartialEq)]
struct PathTo {
    position: Coord,
    cost: u64
}

impl PartialOrd<Self> for PathTo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PathTo {
    fn cmp(&self, other: &Self) -> Ordering {
        Reverse(self.cost).cmp(&Reverse(other.cost)).then(self.position.cmp(&other.position))
    }
}

fn shortest_path_to_end(grid: &Grid<Tile>, time: u64) -> Option<u64> {
    let mut positions_cost = Grid::<u64>::new_with_values(grid.columns(), grid.rows(), u64::MAX);
    let mut unvisited: BinaryHeap<PathTo> = Default::default();

    let start_path = PathTo{
        position: Coord { x: 0, y: 0 },
        cost: 0
    };
    positions_cost[start_path.position] = start_path.cost;
    unvisited.push(start_path);

    let end = Coord {
        x: grid.columns() as i64 - 1,
        y: grid.rows() as i64 - 1,
    };

    let mut cost_to_end = None;
    while let Some(PathTo{position, cost}) = unvisited.pop() {
        if positions_cost[position] < cost {
            continue;
        }
        if matches!(grid[position], Some(wall_at_time) if wall_at_time <= time) {
            continue;
        }
        if position == end {
            cost_to_end = Some(cost);
            break;
        }

        for offset in DIRECTIONS_CARDINAL {
            let next_coord = position + offset;
            if grid.is_valid(next_coord) {
                let new_cost = cost + 1;
                let cost_ref = &mut positions_cost[next_coord];
                if new_cost < *cost_ref {
                    *cost_ref = new_cost;
                    unvisited.push(PathTo{position: next_coord, cost: new_cost})
                }
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
