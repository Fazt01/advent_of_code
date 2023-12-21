use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::{io, mem};
use std::ops::Index;
use anyhow::{Result, Ok, bail, Context};

struct Puzzle {
    map: Map,
    start: Coord,
}

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
}

#[derive(PartialEq, Copy, Clone, Hash, Eq)]
enum Point {
    Garden,
    Rock,
}

#[derive(Copy, Clone)]
struct Offset {
    x: i64,
    y: i64,
}

#[derive(Clone, Eq, PartialEq, Debug)]
struct PositionsId {
    positions: HashSet<Coord>,
}

impl Hash for PositionsId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // todo: maybe not deterministic hash? (at least in the same program run - don't care about different runs)
        self.positions.iter().collect::<Vec<_>>().hash(state)
    }
}

struct PositionTracker {
    visited_submap_states: HashMap<PositionsId, PositionsId>,
    current_submap_states: HashMap<PositionsId, i64>,
}

impl PositionTracker {
    fn move_from_positions(&mut self, map: &Map) {
        let mut prev_submap_states = HashMap::<PositionsId, i64>::new();
        mem::swap(&mut self.current_submap_states, &mut prev_submap_states);
        for (submap_state, submap_state_count) in prev_submap_states {
            let visited = self.visited_submap_states.get(&submap_state);
            match visited {
                None => {
                    let next_state = move_from_positions(&map, &submap_state.positions);
                    let current_positions_id = PositionsId { positions: submap_state.positions };
                    let next_positions_id = PositionsId { positions: next_state };
                    *self.current_submap_states.entry(next_positions_id.clone()).or_default() += 1;
                    self.visited_submap_states.insert(current_positions_id, next_positions_id);
                }
                Some(next) => {
                    *self.current_submap_states.entry(next.clone()).or_default() += submap_state_count;
                }
            }
        }
    }

    fn count(&self) -> i64 {
        let mut sum = 0;
        for (submap_state, submap_state_count) in &self.current_submap_states {
            sum += submap_state.positions.len() as i64 * submap_state_count;
        }
        sum
    }
}

static UP: Offset = Offset { x: 0, y: -1 };
static LEFT: Offset = Offset { x: -1, y: 0 };
static DOWN: Offset = Offset { x: 0, y: 1 };
static RIGHT: Offset = Offset { x: 1, y: 0 };

fn main() -> Result<()> {
    let puzzle = parse()?;

    // part 1
    let mut positions: HashSet<Coord> = [puzzle.start].into();
    for _ in 0..64 {
        positions = move_from_positions(&puzzle.map, &positions);
    }

    println!("{}", positions.len());

    // part 2
    let mut positions_tracker = PositionTracker {
        visited_submap_states: Default::default(),
        current_submap_states: [(
            PositionsId { positions: HashSet::<Coord>::from([puzzle.start]) },
            1,
        )].into(),
    };

    for _ in 0..64 {
        positions_tracker.move_from_positions(&puzzle.map)
    }

    println!("{}", positions_tracker.count());

    Ok(())
}

fn move_from_positions(map: &Map, positions: &HashSet<Coord>) -> HashSet<Coord> {
    let mut result = HashSet::<Coord>::default();
    for from_coord in positions {
        for direction in [UP, LEFT, DOWN, RIGHT] {
            let to_coord = from_coord.offset(direction);
            // todo: somehow detects "leaks" to other submaps. But don't know how to
            // keep track of "leaks" from multiple neighbors at the same time
            if map.is_valid(&to_coord) && matches!(map.index_coord(&to_coord), Point::Garden) {
                result.insert(to_coord);
            }
        }
    }
    result
}

fn parse() -> Result<Puzzle> {
    let stdin = io::stdin();
    let mut points: Vec<Point> = Vec::new();
    let mut rows: usize = 0;
    let mut columns = 0;
    let mut start: Option<Coord> = None;
    for line in stdin.lines() {
        let line = line?;
        columns = line.chars().count();
        rows += 1;

        for (column, c) in line.chars().enumerate() {
            points.push(match c {
                '.' => Point::Garden,
                '#' => Point::Rock,
                'S' => {
                    start = Some(Coord { x: column as i64, y: rows as i64 - 1 });
                    Point::Garden
                }
                _ => bail!("invalid point")
            });
        }
    }
    Ok(Puzzle {
        map: Map {
            points,
            rows,
            columns,
        },
        start: start.context("no start found")?,
    })
}
