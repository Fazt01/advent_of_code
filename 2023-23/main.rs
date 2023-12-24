use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::io;
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

#[derive(Eq, PartialEq, Debug, Copy, Clone, Hash, Ord, PartialOrd)]
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

#[derive(PartialEq, Copy, Clone, Hash, Eq, Debug)]
enum Point {
    Path,
    Forest,
    Slope(Offset),
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
struct Offset {
    x: i64,
    y: i64,
}

static UP: Offset = Offset { x: 0, y: -1 };
static LEFT: Offset = Offset { x: -1, y: 0 };
static DOWN: Offset = Offset { x: 0, y: 1 };
static RIGHT: Offset = Offset { x: 1, y: 0 };

fn main() -> Result<()> {
    let puzzle = parse()?;

    let l = find_longest(&puzzle);

    println!("{}", l);

    let finish = Coord {
        x: puzzle.map.points[(puzzle.map.rows - 1) * puzzle.map.columns..puzzle.map.points.len()]
            .iter()
            .enumerate()
            .find(|(_, &p)| matches!(p, Point::Path))
            .context("no finish found in last row")?
            .0 as i64,
        y: puzzle.map.rows as i64 - 1,
    };

    let mut crossroads: HashSet<Coord> = [puzzle.start, finish].into();
    for y in 0..puzzle.map.rows {
        for x in 0..puzzle.map.columns {
            let point = puzzle.map.index(x, y);
            if matches!(point, Point::Forest) {
                continue;
            }
            let coord = Coord { x: x as i64, y: y as i64 };
            let mut ways = 0;
            for offset in [LEFT, UP, RIGHT, DOWN] {
                let next_coord = coord.offset(offset);
                if !puzzle.map.is_valid(&next_coord) || matches!(puzzle.map.index_coord(&next_coord), Point::Forest) {
                    continue;
                }
                ways += 1;
            }
            if ways >= 3 {
                crossroads.insert(coord);
            }
        }
    }

    let mut from_crossroad: HashMap<Coord, Vec<(Coord, i64)>> = Default::default();

    for coord in &crossroads {
        for (neighbor_crossroad, distance) in neighboring_crossroads_distances(&crossroads, &puzzle.map, coord) {
            from_crossroad.entry(
                *coord,
            ).or_default().push((neighbor_crossroad, distance));
        }
    }

    println!("nodes: {}", crossroads.len());
    println!("edges: {}", from_crossroad.len());

    let mut max_distance = 0;

    let mut last_attempted_edge_index: Option<usize> = None;
    let mut steps: Vec<(Coord, Option<usize>, i64)> = vec![];
    let mut visited_crossroads: HashSet<Coord> = Default::default();
    let mut current = puzzle.start;
    let mut current_distance = 0;
    let mut total_hike_count: i64 = 0;

    loop {
        let next_index =  if current == finish {
            total_hike_count += 1;
            if current_distance > max_distance {
                max_distance = current_distance;
                println!("next max: {}, currently found {} hikes", max_distance, total_hike_count);
            }
            None
        } else {
            match last_attempted_edge_index {
                None => Some(0),
                Some(last) => {
                    let next = last + 1;
                    if next >= from_crossroad.get(&current).unwrap().len() {
                        None
                    } else {
                        Some(next)
                    }
                },
            }
        };
        match next_index {
            None => match steps.pop() {
                None => break,
                Some(popped) => {
                    visited_crossroads.remove(&current);
                    (current, last_attempted_edge_index, current_distance) = popped;
                }
            }
            Some(next_index) => {
                last_attempted_edge_index = Some(next_index);
                let (next_coord, distance) = from_crossroad.get(&current).unwrap()[next_index];
                if visited_crossroads.contains(&next_coord) {
                    continue;
                }
                steps.push((current, last_attempted_edge_index, current_distance));
                visited_crossroads.insert(current);
                last_attempted_edge_index = None;
                current = next_coord;
                current_distance += distance;
            }
        }
    }

    println!("{}, found {} hikes", max_distance, total_hike_count);

    Ok(())
}

struct Step {
    last_attempted_dir: Option<Offset>,
    coord: Coord,
}

fn neighboring_crossroads_distances(crossroads: &HashSet<Coord>, map: &Map, from_coord: &Coord) -> Vec<(Coord, i64)> {
    let mut last_attempted_dir: Option<Offset> = None;
    let mut steps: Vec<Step> = vec![];
    let mut hike_visited: HashSet<Coord> = Default::default();
    let mut current = *from_coord;
    let mut result = vec![];
    loop {
        let next_dir= if &current != from_coord && crossroads.contains(&current) {
            let hike_len = steps.len() as i64;
            result.push((current, hike_len));
            None
        } else {
            get_next_dir(&last_attempted_dir)
        };
        match next_dir {
            None => match steps.pop() {
                None => break,
                Some(popped) => {
                    hike_visited.remove(&current);
                    last_attempted_dir = popped.last_attempted_dir;
                    current = popped.coord;
                }
            }
            Some(next_dir) => {
                last_attempted_dir = Some(next_dir);
                let next_coord = current.offset(next_dir);
                if !map.is_valid(&next_coord) {
                    continue;
                }
                let next_point = map.index_coord(&next_coord);
                if let Point::Forest = next_point {
                    continue;
                }
                if hike_visited.contains(&next_coord) {
                    continue;
                }
                steps.push(Step {
                    last_attempted_dir,
                    coord: current,
                });
                hike_visited.insert(current);
                last_attempted_dir = None;
                current = next_coord;
            }
        }
    }
    result
}

fn find_longest(puzzle: &Puzzle) -> i64 {
    let mut last_attempted_dir: Option<Offset> = None;
    let mut current = puzzle.start;
    let mut steps: Vec<Step> = vec![];
    let mut hike_visited: HashSet<Coord> = Default::default();
    let mut max_hike_len = 0;
    loop {
        if current.y == puzzle.map.rows as i64 - 1 {
            let hike_len = steps.len() as i64;
            if hike_len > max_hike_len {
                max_hike_len = hike_len
            }
        }
        let next_dir = get_next_dir(&last_attempted_dir);
        match next_dir {
            None => match steps.pop() {
                None => break,
                Some(popped) => {
                    hike_visited.remove(&current);
                    last_attempted_dir = popped.last_attempted_dir;
                    current = popped.coord;
                }
            }
            Some(next_dir) => {
                last_attempted_dir = Some(next_dir);
                if let Point::Slope(slope_dir) = puzzle.map.index_coord(&current) {
                    if slope_dir != &next_dir {
                        continue;
                    }
                }
                let next_coord = current.offset(next_dir);
                if !puzzle.map.is_valid(&next_coord) {
                    continue;
                }
                let next_point = puzzle.map.index_coord(&next_coord);
                if let Point::Forest = next_point {
                    continue;
                }
                if hike_visited.contains(&next_coord) {
                    continue;
                }
                steps.push(Step {
                    last_attempted_dir,
                    coord: current,
                });
                hike_visited.insert(current);
                last_attempted_dir = None;
                current = next_coord;
            }
        }
    }
    max_hike_len
}

fn get_next_dir(offset: &Option<Offset>) -> Option<Offset> {
    match offset {
        None => Some(LEFT),
        Some(prev_direction) => match prev_direction {
            _ if &LEFT == prev_direction => Some(UP),
            _ if &UP == prev_direction => Some(RIGHT),
            _ if &RIGHT == prev_direction => Some(DOWN),
            _ if &DOWN == prev_direction => None,
            _ => unreachable!(),
        }
    }
}

fn parse() -> Result<Puzzle> {
    let stdin = io::stdin();
    let mut points: Vec<Point> = Vec::new();
    let mut rows: usize = 0;
    let mut columns = 0;
    let mut start: Option<Coord> = None;
    for (row, line) in stdin.lines().enumerate() {
        let line = line?;
        columns = line.chars().count();
        rows += 1;

        for (column, c) in line.chars().enumerate() {
            points.push(match c {
                '.' => {
                    if row == 0 {
                        start = Some(Coord {
                            x: column as i64,
                            y: row as i64,
                        })
                    }
                    Point::Path
                }
                '#' => Point::Forest,
                _ => Point::Slope(match c {
                    '>' => RIGHT,
                    'v' => DOWN,
                    '<' => LEFT,
                    '^' => UP,
                    _ => bail!("invalid point"),
                }),
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