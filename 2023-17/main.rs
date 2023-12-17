use std::cmp::Reverse;
use std::collections::HashMap;
use std::io;
use std::ops::Index;
use anyhow::{Result, Ok, Context};

struct Map {
    points: Vec<Point>,
    rows: usize,
    columns: usize,
}

impl Map {
    fn index(&self, x: usize, y: usize) -> &Point {
        self.points.index(y * self.columns + x)
    }

    fn index_coord(&self, coord: &Coord) -> &Point {
        self.index(coord.x as usize, coord.y as usize)
    }

    fn is_valid(&self, coord: &Coord) -> bool {
        coord.x >= 0 && coord.y >= 0 && coord.x < self.columns as i64 && coord.y < self.rows as i64
    }

    fn shortest_path(&self, from: Coord, to: Coord, min_to_turn: u8, max_to_straight: u8) -> Option<PathState> {
        let mut best_next_states = vec![PathState {
            coord: from,
            last_move: RIGHT,
            total_heat_loss: 0,
            optimistic_total_heat_loss_estimate: from.abs_distance(to),
            straight_moves: None,
        }];

        let mut already_visited = HashMap::<Visited, u64>::new();

        while let Some(ref best_next) = best_next_states.pop() {
            let visited = Visited {
                coord: best_next.coord,
                last_move: best_next.last_move,
                straight_moves: best_next.straight_moves,
            };
            let old = already_visited.get(&visited);

            if let Some(&old) = old {
                if old <= best_next.total_heat_loss {
                    // this state is already worse than some that have been tried
                    continue;
                }
            }
            already_visited.insert(visited, best_next.total_heat_loss);

            let mut directions: Vec<Offset> = Vec::new();

            // can turn or stop?
            if match best_next.straight_moves {
                None => true,
                Some(moves) => moves >= min_to_turn
            } {
                if best_next.coord == to {
                    return Some(*best_next);
                }
                directions.push(best_next.last_move.rotate(ROTATE_LEFT));
                directions.push(best_next.last_move.rotate(ROTATE_RIGHT));
            }

            // can go straight?
            if match best_next.straight_moves {
                None => true,
                Some(moves) => moves < max_to_straight
            } {
                directions.push(best_next.last_move);
            }

            for direction in directions {
                let new_coord = best_next.coord.offset(direction);
                if !self.is_valid(&new_coord) {
                    continue;
                }

                let new_total_heat_loss = best_next.total_heat_loss + self.index_coord(&new_coord).heat_loss;
                let new_estimate = new_total_heat_loss + new_coord.abs_distance(to);
                let new_state = PathState {
                    coord: new_coord,
                    last_move: direction,
                    total_heat_loss: new_total_heat_loss,
                    optimistic_total_heat_loss_estimate: new_estimate,
                    straight_moves: Some(
                        if direction == best_next.last_move {
                            best_next.straight_moves.unwrap_or(0) + 1
                        } else {
                            1
                        }
                    ),
                };

                let search_result = best_next_states.binary_search_by_key(
                    &Reverse(new_estimate),
                    |x| Reverse(x.optimistic_total_heat_loss_estimate),
                );

                let pos = match search_result {
                    Result::Ok(i) => i,
                    Err(i) => i,
                };

                best_next_states.insert(pos, new_state);
            }
        }

        None
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
struct Visited {
    coord: Coord,
    last_move: Offset,
    straight_moves: Option<u8>,
}

#[derive(Copy, Clone)]
struct PathState {
    coord: Coord,
    last_move: Offset,
    total_heat_loss: u64,
    optimistic_total_heat_loss_estimate: u64,
    straight_moves: Option<u8>,
}

#[derive(Eq, PartialEq, Debug, Copy, Clone, Hash)]
struct Coord {
    x: i64,
    y: i64,
}

impl Coord {
    fn offset(self, offset: Offset) -> Coord {
        Coord {
            x: self.x + offset.x,
            y: self.y + offset.y,
        }
    }

    fn abs_distance(&self, other: Coord) -> u64 {
        self.x.abs_diff(other.x) + self.y.abs_diff(other.y)
    }
}

#[derive(Copy, Clone)]
struct Point {
    heat_loss: u64,
}

impl Point {
    fn from_char(value: char) -> Option<Point> {
        let digit = value.to_digit(10)?;
        Some(Point {
            heat_loss: digit as u64,
        })
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
struct Offset {
    x: i64,
    y: i64,
}

impl Offset {
    fn rotate(self, rotation: Rotation) -> Offset {
        Offset {
            x: self.y * rotation.x as i64,
            y: self.x * rotation.y as i64,
        }
    }
}

static RIGHT: Offset = Offset { x: 1, y: 0 };

#[derive(Copy, Clone)]
struct Rotation {
    x: i8,
    y: i8,
}

static ROTATE_RIGHT: Rotation = Rotation { x: -1, y: 1 };
static ROTATE_LEFT: Rotation = Rotation { x: 1, y: -1 };

fn main() -> Result<()> {
    let map = parse()?;

    let least_heat_loss = map.shortest_path(
        Coord { x: 0, y: 0 },
        Coord { x: map.columns as i64 - 1, y: map.rows as i64 - 1 },
        // part 1
        // 1,
        // 3,
        // part 2
        4,
        10,
    );

    println!("{}", least_heat_loss.unwrap().total_heat_loss);

    Ok(())
}

fn parse() -> Result<Map> {
    let stdin = io::stdin();
    let mut points: Vec<Point> = Vec::new();
    let mut rows = 0;
    let mut columns = 0;
    for line in stdin.lines() {
        let line = line?;
        columns = line.chars().count();
        rows += 1;

        for c in line.chars() {
            points.push(Point::from_char(c).context("invalid char")?);
        }
    }
    Ok(Map {
        points,
        rows,
        columns,
    })
}
