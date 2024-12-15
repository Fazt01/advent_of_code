use anyhow::{bail, Context, Result};
use grid::{Coord, Grid, Offset, OFFSET_DOWN, OFFSET_LEFT, OFFSET_RIGHT, OFFSET_UP};
use owned_chars::OwnedCharsExt;
use std::io::{stdin, BufRead, Read};

struct Puzzle {
    grid: Grid<Tile>,
    position: Coord,
    move_plan: Vec<Offset>,
}

#[derive(Default, Clone, Copy)]
enum Tile {
    Wall,
    // left part of box for part 2
    Box,
    #[default]
    Empty,
}

fn main() -> Result<()> {
    let mut input = parse_input()?;
    let mut bigger_input = make_it_bigger(&input);

    for &offset in &input.move_plan {
        // part 1
        let next = input.position + offset;
        match input.grid[next] {
            Tile::Wall => {}
            Tile::Box => {
                if move_box(&mut input.grid, next, offset) {
                    input.position = next;
                }
            }
            Tile::Empty => {
                input.position = next;
            }
        }

        // part 2
        let next = bigger_input.position + offset;
        match bigger_input.grid[next] {
            Tile::Wall => {}
            Tile::Box => {
                if move_box2(&mut bigger_input.grid, next, offset, false) {
                    move_box2(&mut bigger_input.grid, next, offset, true);
                    bigger_input.position = next;
                }
            }
            Tile::Empty => {
                // check if robot hits right side of box
                let next_left = next + OFFSET_LEFT;
                if matches!(bigger_input.grid[next_left], Tile::Box) {
                    if move_box2(&mut bigger_input.grid, next_left, offset, false) {
                        move_box2(&mut bigger_input.grid, next_left, offset, true);
                        bigger_input.position = next;
                    }
                } else {
                    bigger_input.position = next;
                }
            }
        }
    }

    println!("{}", calc_grid_gps_sum(&input.grid));
    println!("{}", calc_grid_gps_sum(&bigger_input.grid));

    Ok(())
}

fn move_box(grid: &mut Grid<Tile>, coord: Coord, offset: Offset) -> bool {
    let next = coord + offset;
    match grid[next] {
        Tile::Wall => false,
        Tile::Box => {
            let moved = move_box(grid, next, offset);
            if moved {
                grid.swap(coord, next);
            }
            moved
        }
        Tile::Empty => {
            grid.swap(coord, next);
            true
        }
    }
}

fn move_box2(grid: &mut Grid<Tile>, coord: Coord, offset: Offset, do_move: bool) -> bool {
    let next = coord + offset;
    match grid[next] {
        Tile::Wall => false,
        Tile::Box => {
            // left side of current box hits left side of next box
            let moved = move_box2(grid, next, offset, do_move);
            if moved && do_move {
                grid.swap(coord, next);
            }
            moved
        }
        Tile::Empty => {
            // check for right side of current box, that can hit the left side of next box
            let next_right = next + OFFSET_RIGHT;
            let right_moved = match grid[next_right] {
                Tile::Wall => false,
                // short-circuit to avoid collision of current box left and right sides
                Tile::Box => next_right == coord || move_box2(grid, next_right, offset, do_move),
                Tile::Empty => true,
            };
            // check for left side of current box, that can hit the right side of next box
            let next_left = next + OFFSET_LEFT;
            let left_moved = match grid[next_left] {
                Tile::Wall => true,
                // short-circuit to avoid collision of current box left and right sides
                Tile::Box => next_left == coord || move_box2(grid, next_left, offset, do_move),
                Tile::Empty => true,
            };
            if right_moved && left_moved && do_move {
                grid.swap(coord, next);
            }
            right_moved && left_moved
        }
    }
}

fn calc_gps(coord: Coord) -> i64 {
    coord.x + 100 * coord.y
}

fn calc_grid_gps_sum(grid: &Grid<Tile>) -> i64 {
    grid
        .iter()
        .filter(|(_, x)| matches!(x, Tile::Box))
        .map(|(coord, _)| calc_gps(coord))
        .sum::<i64>()
}

fn make_it_bigger(puzzle: &Puzzle) -> Puzzle {
    let mut new_puzzle = Puzzle {
        grid: Grid::new(puzzle.grid.columns() * 2, puzzle.grid.rows()),
        position: Coord {
            x: puzzle.position.x * 2,
            y: puzzle.position.y,
        },
        move_plan: puzzle.move_plan.clone(),
    };
    for (coord, &tile) in puzzle.grid.iter() {
        let left_coord = Coord {
            x: coord.x * 2,
            y: coord.y,
        };
        let right_coord = left_coord + OFFSET_RIGHT;
        new_puzzle.grid[left_coord] = tile;
        new_puzzle.grid[right_coord] = match tile {
            Tile::Wall => Tile::Wall,
            Tile::Box => Tile::Empty,
            Tile::Empty => Tile::Empty,
        }
    }
    new_puzzle
}

fn parse_input() -> Result<Puzzle> {
    let mut position: Option<Coord> = None;
    let mut stdin = stdin().lock();
    let grid = Grid::from_lines_try_iter_map(
        (&mut stdin)
            .lines()
            .take_while(|line| !matches!(line, Ok(line) if line.is_empty()))
            .map(|line| -> Result<_> {
                let line = line?;
                Ok(line.into_chars())
            }),
        |coord, c| {
            Ok(match c {
                '#' => Tile::Wall,
                'O' => Tile::Box,
                '.' => Tile::Empty,
                '@' => {
                    if position.is_some() {
                        bail!("found duplicate robot position")
                    }
                    position = Some(coord);
                    Tile::Empty
                }
                _ => bail!("unexpected character for map"),
            })
        },
    )?;

    let mut buf = String::new();
    stdin.read_to_string(&mut buf)?;
    let move_plan = buf
        .chars()
        .filter_map(|c| match c {
            '>' => Some(OFFSET_RIGHT),
            'v' => Some(OFFSET_DOWN),
            '<' => Some(OFFSET_LEFT),
            '^' => Some(OFFSET_UP),
            _ => None,
        })
        .collect();

    Ok(Puzzle {
        grid,
        position: position.context("robot position not found")?,
        move_plan,
    })
}
