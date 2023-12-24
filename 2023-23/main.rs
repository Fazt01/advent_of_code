use std::cell::RefCell;
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

    let visited_crossroads: RefCell<HashSet<Coord>> = Default::default();
    let mut total_hike_count: i64 = 0;

    dfs(
        (puzzle.start, 0),
        |to_expand| {
            visited_crossroads.borrow_mut().insert(to_expand.0);
            if to_expand.0 == finish {
                total_hike_count += 1;
                if to_expand.1 > max_distance {
                    max_distance = to_expand.1;
                    println!("next max: {}, currently found {} hikes", max_distance, total_hike_count);
                }
                return [].into();
            }
            let expanded_coords = from_crossroad.get(&to_expand.0).unwrap();
            expanded_coords
                .iter()
                .filter(|c| !visited_crossroads.borrow().contains(&c.0))
                .map(|&(coord, distance)| (coord, to_expand.1 + distance))
                .collect()
        }, |to_revert| {
            visited_crossroads.borrow_mut().remove(&to_revert.0);
        },
    );

    println!("{}, found {} hikes", max_distance, total_hike_count);

    Ok(())
}

fn neighboring_crossroads_distances(crossroads: &HashSet<Coord>, map: &Map, from_coord: &Coord) -> Vec<(Coord, i64)> {
    #[derive(Copy, Clone)]
    struct State {
        coord: Coord,
        distance: i64,
    }

    let hike_visited: RefCell<HashSet<Coord>> = Default::default();
    let mut result = vec![];
    dfs(State {
        coord: *from_coord,
        distance: 0,
    }, |to_expand| {
        hike_visited.borrow_mut().insert(to_expand.coord);
        if to_expand.coord != *from_coord && crossroads.contains(&to_expand.coord) {
            result.push((to_expand.coord, to_expand.distance));
            return [].into();
        };
        let point = map.index_coord(&to_expand.coord);
        let expand_dirs = match point {
            Point::Path | Point::Slope(_) => vec![LEFT, UP, RIGHT, DOWN],
            Point::Forest => vec![],
        };
        expand_dirs
            .iter()
            .map(|&offset| to_expand.coord.offset(offset))
            .filter(|c| map.is_valid(c) && !hike_visited.borrow().contains(c))
            .map(|c| State {
                coord: c,
                distance: to_expand.distance + 1,
            })
            .collect()
    }, |to_revert| {
        hike_visited.borrow_mut().remove(&to_revert.coord);
    });

    result
}

fn find_longest(puzzle: &Puzzle) -> i64 {
    #[derive(Copy, Clone)]
    struct State {
        coord: Coord,
        distance: i64,
    }

    let hike_visited: RefCell<HashSet<Coord>> = Default::default();
    let mut max_hike_len = 0;
    dfs(State {
        coord: puzzle.start,
        distance: 0,
    }, |to_expand| {
        hike_visited.borrow_mut().insert(to_expand.coord);
        if to_expand.coord.y == puzzle.map.rows as i64 - 1 {
            let hike_len = to_expand.distance;
            if hike_len > max_hike_len {
                max_hike_len = hike_len
            }
            return [].into();
        };
        let point = puzzle.map.index_coord(&to_expand.coord);
        let expand_dirs = match point {
            Point::Path => vec![LEFT, UP, RIGHT, DOWN],
            Point::Forest => vec![],
            Point::Slope(slope_dir) => vec![*slope_dir],
        };
        expand_dirs
            .iter()
            .map(|&offset| to_expand.coord.offset(offset))
            .filter(|c| puzzle.map.is_valid(c) && !hike_visited.borrow().contains(c))
            .map(|c| State {
                coord: c,
                distance: to_expand.distance + 1,
            })
            .collect()
    }, |to_revert| {
        hike_visited.borrow_mut().remove(&to_revert.coord);
    });

    max_hike_len
}

fn dfs<TState, TExpandFn, TRevertFn>(
    init_state: TState,
    mut expand_fn: TExpandFn,
    mut revert_global_state_fn: TRevertFn,
) where
    TState: Clone,
    TExpandFn: FnMut(TState) -> Box<[TState]>,
    TRevertFn: FnMut(TState)
{
    enum StackItem<TState> {
        Expanded(TState),
        Revert(TState),
    }

    let mut stack = vec![StackItem::Expanded(init_state)];
    while let Some(popped) = stack.pop() {
        match popped {
            StackItem::Expanded(popped) => {
                stack.push(StackItem::Revert(popped.clone()));
                stack.append(&mut Vec::from(expand_fn(popped))
                    .into_iter()
                    .map(|x| StackItem::Expanded(x))
                    .collect()
                );
            }
            StackItem::Revert(popped) => {
                revert_global_state_fn(popped)
            }
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