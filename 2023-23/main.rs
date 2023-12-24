use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
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

#[derive(Eq, PartialEq, Hash, Clone)]
struct SubHikeKey {
    start: Coord,
    end: Coord,
    visited: VisitedCrossroads,
}

#[derive(Eq, PartialEq, Clone)]
struct VisitedCrossroads(HashSet<Coord>);

impl Hash for VisitedCrossroads {
    fn hash<H: Hasher>(&self, _: &mut H) {
        // let mut v = self.0.iter().collect::<Vec<_>>();
        // v.sort();
        // v.hash(state);
    }
}

fn main() -> Result<()> {
    let puzzle = parse()?;

    let l = find_longest(&puzzle);

    println!("{}", l);

    let finish = Coord {
        x: puzzle.map.points[(puzzle.map.rows - 1) * puzzle.map.columns..puzzle.map.points.len()]
            .iter()
            .enumerate()
            .filter(|(_, &p)| matches!(p, Point::Path))
            .next()
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

    let mut sub_hikes: HashMap<SubHikeKey, i64> = Default::default();

    for coord in &crossroads {
        for (neighbor_crossroad, distance) in neighboring_crossroads_distances(&crossroads, &puzzle.map, coord) {
            let visited = VisitedCrossroads([*coord, neighbor_crossroad].into());
            sub_hikes.insert(
                SubHikeKey{
                    start: *coord,
                    end: neighbor_crossroad,
                    visited: visited.clone(),
                },
                distance,
            );
        }
    }

    println!("nodes: {}", crossroads.len());
    println!("edges: {}", sub_hikes.len());

    let sub_hikes = sub_hikes;
    let mut super_hikes: HashMap<SubHikeKey, i64> = [(SubHikeKey{
        start: puzzle.start,
        end: puzzle.start,
        visited: VisitedCrossroads([puzzle.start].into()),
    }, 0)].into();

    for i in 0.. {
        let start = std::time::Instant::now();

        let mut new_super_hikes: HashMap<SubHikeKey, i64> = Default::default();

        for (sub_hike1, &distance1) in &super_hikes {
            for (sub_hike2, &distance2) in &sub_hikes {
                if sub_hike1.visited.0.intersection(&sub_hike2.visited.0).count() != 1 {
                    continue;
                }
                let (new_start, new_end) = if sub_hike1.end == sub_hike2.start {
                    (sub_hike1.start, sub_hike2.end)
                } else {
                    continue
                };
                let new_visited = sub_hike1.visited.0
                    .union(&sub_hike2.visited.0)
                    .map(|&x| x)
                    .collect::<HashSet<Coord>>();
                let new_key = SubHikeKey{
                    start: new_start,
                    end: new_end,
                    visited: VisitedCrossroads(new_visited),
                };
                let new_distance = distance1 + distance2;
                new_super_hikes.entry(new_key).and_modify(|distance| {
                    if *distance < new_distance {
                        *distance = new_distance;
                    }
                }).or_insert(new_distance);
            }
        }

        if new_super_hikes.is_empty() {
            break;
        }

        super_hikes = new_super_hikes;
        println!("iteration {i} done in {:?} with {} hikes", start.elapsed(), super_hikes.len());
    }

    let mut max_distance = 0;

    for (subhike, &distance) in &super_hikes {
        if subhike.visited.0.contains(&puzzle.start) && subhike.visited.0.contains(&finish) {
            if max_distance < distance {
                max_distance = distance
            }
        }
    }

    println!("{}", max_distance);

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
        let next_dir;
        if &current != from_coord && crossroads.contains(&current) {
            let hike_len = steps.len() as i64;
            result.push((current, hike_len));
            next_dir = None
        } else {
            next_dir = get_next_dir(&last_attempted_dir);
        }
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