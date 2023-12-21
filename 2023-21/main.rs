use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::io;
use std::ops::{Add, Index};
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

    // returns a coord that is valid in current Map. If input is not valid,
    // it will be wrapped into valid one, with result Offset representing number of "submaps" moved
    // in specified directions.
    fn mod_coord(&self, coord: &Coord) -> (Coord, Offset) {
        let (x_div, x_rem) = modulo(coord.x, self.columns as i64);
        let (y_div, y_rem) = modulo(coord.y, self.rows as i64);
        (Coord {
            x: x_rem,
            y: y_rem,
        }, Offset {
            x: x_div,
            y: y_div,
        })
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

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
struct Offset {
    x: i64,
    y: i64,
}

impl Add for Offset {
    type Output = Offset;

    fn add(self, rhs: Self) -> Self::Output {
        Offset {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

fn modulo(num: i64, divisor: i64) -> (i64, i64) {
    let mut div = num / divisor;
    let mut rem = num % divisor;
    if rem < 0 {
        div -= 1;
        rem += divisor;
    }
    (div, rem)
}

static UP: Offset = Offset { x: 0, y: -1 };
static LEFT: Offset = Offset { x: -1, y: 0 };
static DOWN: Offset = Offset { x: 0, y: 1 };
static RIGHT: Offset = Offset { x: 1, y: 0 };

fn main() -> Result<()> {
    let puzzle = parse()?;

    // part 1
    let reachable_exactly_in: i64 = 64;
    let mut positions: HashSet<Coord> = [puzzle.start].into();
    for _ in 0..reachable_exactly_in {
        positions = move_from_positions(&puzzle.map, &positions);
    }

    println!("{}", positions.len());

    // part 2
    let reachable_exactly_in: i64 = 26501365;
    assert_eq!(puzzle.map.rows, puzzle.map.columns);
    let across = puzzle.map.rows as i64;
    assert_eq!(across % 2, 1);
    let mid = across / 2;
    assert_eq!(puzzle.start, Coord { x: mid, y: mid });

    let mut extended_positions_reachability: HashMap<Offset, HashMap<Coord, i64>> = Default::default();
    for offset in [
        Offset { x: 0, y: 0 },
        UP,
        UP + RIGHT,
        RIGHT,
        RIGHT + DOWN,
        DOWN,
        DOWN + LEFT,
        LEFT,
        LEFT + UP,
    ] {
        extended_positions_reachability.insert(offset, Default::default());
        if offset.x == 0 && offset.y == 0 {
            extended_positions_reachability.get_mut(&offset).unwrap().insert(puzzle.start, 0);
        }
    }

    for i in 0.. {
        let prev_extended_positions_reachability: HashMap<Offset, HashMap<Coord, i64>> = extended_positions_reachability.clone();
        for (offset, offset_reachability) in prev_extended_positions_reachability.clone() {
            let mut from_coords: HashSet<Coord> = Default::default();
            for (reachable, reachable_in) in offset_reachability {
                if reachable_in != i {
                    continue;
                }
                from_coords.insert(reachable);
            }
            let unbounded_to_coords = move_from_positions_unbounded(&puzzle.map, &from_coords);
            for reached in unbounded_to_coords {
                let (wrapped_coord, wrapped_offset) = puzzle.map.mod_coord(&reached);
                let sum_offset = offset + wrapped_offset;
                if sum_offset.x.abs() >= 2 || sum_offset.y.abs() >= 2 {
                    continue;
                }
                extended_positions_reachability.get_mut(&sum_offset).unwrap().entry(wrapped_coord).or_insert(i + 1);
            }
        }
        if prev_extended_positions_reachability == extended_positions_reachability {
            break;
        }
    }

    let mut sum = 0;
    // center submap
    for &reachable_in in extended_positions_reachability.get(&Offset { x: 0, y: 0 }).unwrap().values() {
        if reachable_in <= reachable_exactly_in && (reachable_exactly_in - reachable_in) % 2 == 0 {
            sum += 1
        }
    }
    // in cardinal directions
    for offset in [UP, RIGHT, DOWN, LEFT] {
        for &reachable_in in extended_positions_reachability.get(&offset).unwrap().values() {
            if reachable_in > reachable_exactly_in {
                continue;
            }
            let diff = reachable_exactly_in - reachable_in;

            sum += cardinal_count(diff, across);
        }
    }
    // in diagonal directions
    for offset in [UP + RIGHT, RIGHT + DOWN, DOWN + LEFT, LEFT + UP] {
        for &reachable_in in extended_positions_reachability.get(&offset).unwrap().values() {
            if reachable_in > reachable_exactly_in {
                continue;
            }
            let diff = reachable_exactly_in - reachable_in;
            sum += diagonal_count(diff, across);
        }
    }

    println!("{}", sum);

    Ok(())
}

fn move_from_positions(map: &Map, positions: &HashSet<Coord>) -> HashSet<Coord> {
    move_from_positions_unbounded(map, positions).into_iter().filter(|x| map.is_valid(x)).collect()
}

fn move_from_positions_unbounded(map: &Map, positions: &HashSet<Coord>) -> HashSet<Coord> {
    let mut result = HashSet::<Coord>::default();
    for from_coord in positions {
        for direction in [UP, LEFT, DOWN, RIGHT] {
            let to_coord = from_coord.offset(direction);
            if !map.is_valid(&to_coord) || matches!(map.index_coord(&to_coord), Point::Garden) {
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

fn cardinal_count(diff: i64, across: i64) -> i64 {
    // max number of submaps this point is reachable in that direction
    // S01234
    let reachable_in_submap_count = diff / across;
    let parity = (reachable_in_submap_count + diff) % 2;
    ((reachable_in_submap_count + parity) / 2) + ((parity + 1) % 2)
}

fn diagonal_count(diff: i64, across: i64) -> i64 {
    // number this point is reachable in that diagonal direction
    // .43456
    // .34567
    // .23456
    // .12345
    // .01234
    // S....
    let reachable_in_diagonal = diff / across;
    let parity = (reachable_in_diagonal + diff % across) % 2;

    // let reachable_in_diagonal be n;
    //
    // for parity 0:
    // result = 1 + 3 + 5 + 7 ... (n/2+1 items)
    // let x = n/2+1
    // result = 1 + 3 + 5 + 7 ... (x items)
    // result = -x + (2 + 4 + 6 + 8 ... (x items))
    // result = -x + 2 * (x * (x+1) / 2)
    // result = -x + (x * (x + 1))
    // result = -x + (x * (x + 1))
    // result = x * x
    //
    // for parity 1:
    // result = 2 + 4 + 6 + 8 ... ((n+1)/2 items)
    // result = 2 * (1 + 2 + 3 + 4 ... ((n+1)/2 items)))
    // let x = (n+1)/2
    // result = 2 * (x * (x+1) / 2)
    // result = x * (x+1)
    //
    match parity {
        0 => {
            let x = reachable_in_diagonal / 2 + 1;
            x * x
        }
        1 => {
            let x = (reachable_in_diagonal+1) / 2;
            x * (x + 1)
        }
        _ => unreachable!()
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use crate::{cardinal_count, diagonal_count};

    #[rstest]
    #[case(0, 1, 1)]
    #[case(1, 1, 1)]
    #[case(2, 1, 2)]
    #[case(3, 1, 2)]
    #[case(4, 1, 3)]
    #[case(5, 1, 3)]
    #[case(0, 3, 1)]
    #[case(1, 3, 0)]
    #[case(2, 3, 1)]
    #[case(3, 3, 1)]
    #[case(4, 3, 1)]
    #[case(5, 3, 1)]
    #[case(6, 3, 2)]
    #[case(7, 3, 1)]
    #[case(0, 131, 1)]
    #[case(131, 131, 1)]
    #[case(132, 131, 1)]
    #[case(262, 131, 2)]
    #[case(263, 131, 1)]
    fn test_cardinal_count(#[case] diff: i64, #[case] across: i64, #[case] expected: i64) {
        assert_eq!(cardinal_count(diff, across), expected)
    }

    #[rstest]
    #[case(0, 1, 1)]
    #[case(1, 1, 2)]
    #[case(2, 1, 4)]
    #[case(3, 1, 6)]
    #[case(4, 1, 9)]
    #[case(5, 1, 12)]
    #[case(0, 3, 1)]
    #[case(1, 3, 0)]
    #[case(2, 3, 1)]
    #[case(3, 3, 2)]
    #[case(4, 3, 1)]
    #[case(5, 3, 2)]
    #[case(6, 3, 4)]
    #[case(7, 3, 2)]
    #[case(0, 131, 1)]
    #[case(131, 131, 2)]
    #[case(132, 131, 1)]
    #[case(262, 131, 4)]
    #[case(263, 131, 2)]
    fn test_diagonal_count(#[case] diff: i64, #[case] across: i64, #[case] expected: i64) {
        assert_eq!(diagonal_count(diff, across), expected)
    }
}