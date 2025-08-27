use anyhow::{bail, Context, Result};
use lib::grid::{Coord, Grid, Offset, DIRECTIONS_CARDINAL, OFFSET_RIGHT};
use owned_chars::OwnedCharsExt;
use std::collections::{HashMap, HashSet};
use std::io::stdin;

struct Puzzle {
    grid: Grid<Tile>,
    start: Coord,
    end: Coord,
}

#[derive(Default, Clone, Copy)]
enum Tile {
    Wall,
    #[default]
    Empty,
}

fn main() -> Result<()> {
    let input = parse_input()?;

    let mut unvisited: HashMap<(Coord, Offset), (u64, HashSet<Coord>)> = Default::default();

    for (coord, &tile) in input.grid.iter() {
        if matches!(tile, Tile::Empty) {
            for offset in DIRECTIONS_CARDINAL {
                unvisited.insert((coord, offset), (u64::MAX, Default::default()));
            }
        }
    }

    unvisited.insert((input.start, OFFSET_RIGHT), (0, HashSet::from([input.start])));

    let mut cost_to_end = None;
    let mut best_paths_visited = None;
    while !unvisited.is_empty() {
        let mut min_offset = Offset::default();
        let mut min_coord = Coord::default();
        let mut min = u64::MAX;
        for (&(coord, offset), &(cost, _)) in &unvisited {
            if cost < min {
                min = cost;
                min_coord = coord;
                min_offset = offset;
            }
        }

        let (_, mut best_visited_nodes) = unvisited.remove(&(min_coord, min_offset)).unwrap();
        best_visited_nodes.insert(min_coord);
        if min_coord == input.end {
            cost_to_end = Some(min);
            best_paths_visited = Some(best_visited_nodes.clone())
        }
        if let Some(max_cost) = cost_to_end {
           if max_cost < min {
               break
           }
        }

        // move forward
        let next_coord = min_coord + min_offset;
        if input.grid.is_valid(next_coord) && matches!(input.grid[next_coord], Tile::Empty) {
            let new_cost = min + 1;
            update_unvisited_entry(&mut unvisited, next_coord, min_offset, new_cost, &best_visited_nodes);
        }

        // turn left
        let new_cost = min + 1000;
        update_unvisited_entry(&mut unvisited, min_coord, min_offset.rotate_left(), new_cost, &best_visited_nodes);

        // turn right
        update_unvisited_entry(&mut unvisited, min_coord, min_offset.rotate_right(), new_cost, &best_visited_nodes);
    }

    println!("{}", cost_to_end.context("end tile not reachable")?);
    println!("{}", best_paths_visited.context("end tile not reachable")?.len());

    Ok(())
}

fn update_unvisited_entry(unvisited: &mut HashMap<(Coord, Offset), (u64, HashSet<Coord>)>, coord: Coord, offset: Offset, cost: u64, from_tiles: &HashSet<Coord>) {
    unvisited.get_mut(&(coord, offset)).map(|(cost_so_far, visited_tiles)| {
        if cost < *cost_so_far {
            *cost_so_far = cost;
            *visited_tiles = from_tiles.clone();
        } else if cost == *cost_so_far {
            visited_tiles.extend(from_tiles)
        }
    });
}

fn parse_input() -> Result<Puzzle> {
    let mut start: Option<Coord> = None;
    let mut end: Option<Coord> = None;
    let grid = Grid::from_lines_try_iter_map(
        stdin()
            .lines()
            .take_while(|line| !matches!(line, Ok(line) if line.is_empty()))
            .map(|line| -> Result<_> {
                let line = line?;
                Ok(line.into_chars())
            }),
        |coord, c| {
            Ok(match c {
                '#' => Tile::Wall,
                '.' => Tile::Empty,
                'E' => {
                    if end.is_some() {
                        bail!("found duplicate end position")
                    }
                    end = Some(coord);
                    Tile::Empty
                }
                'S' => {
                    if start.is_some() {
                        bail!("found duplicate start position")
                    }
                    start = Some(coord);
                    Tile::Empty
                }
                _ => bail!("unexpected character for map"),
            })
        },
    )?;

    Ok(Puzzle {
        grid,
        start: start.context("no start found")?,
        end: end.context("no end found")?
    })
}
