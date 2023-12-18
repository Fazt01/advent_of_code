use std::cmp::Ordering;
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::io;
use std::ops::{Index, IndexMut, Mul, Neg};
use anyhow::{Result, Ok, bail, Context};
use once_cell::sync::Lazy;
use regex::Regex;


struct Map {
    points: Vec<Point>,
    rows: usize,
    columns: usize,
}

impl Display for Map {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.rows {
            for x in 0..self.columns {
                write!(f, "{}", match self.index(x, y) {
                    Point::Ground => '.',
                    Point::Dig => '#',
                })?;
            }
            writeln!(f)?;
        }
        Result::Ok(())
    }
}

impl Map {
    fn index(&self, x: usize, y: usize) -> &Point {
        self.points.index(y * self.columns + x)
    }

    fn index_mut(&mut self, x: usize, y: usize) -> &mut Point {
        self.points.index_mut(y * self.columns + x)
    }

    fn index_coord_mut(&mut self, coord: &Coord) -> &mut Point {
        self.index_mut(coord.x as usize, coord.y as usize)
    }

    fn is_valid(&self, coord: &Coord) -> bool {
        coord.x >= 0 && coord.y >= 0 && coord.x < self.columns as i64 && coord.y < self.rows as i64
    }
}

#[derive(Debug)]
enum Point {
    Ground,
    Dig,
}

#[derive(Copy, Clone, PartialEq, Debug)]
struct Offset {
    x: i64,
    y: i64,
}

impl Mul<i64> for Offset {
    type Output = Offset;

    fn mul(self, rhs: i64) -> Self::Output {
        Offset {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Neg for Offset {
    type Output = Offset;

    fn neg(self) -> Self::Output {
        Offset {
            x: -self.x,
            y: -self.y,
        }
    }
}

static UP: Offset = Offset { x: 0, y: -1 };
static LEFT: Offset = Offset { x: -1, y: 0 };
static DOWN: Offset = Offset { x: 0, y: 1 };
static RIGHT: Offset = Offset { x: 1, y: 0 };

#[derive(Copy, Clone, PartialEq, Debug, Hash, Eq)]
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

#[derive(Copy, Clone)]
struct Rotation {
    x: i8,
    y: i8,
}

static ROTATE_RIGHT: Rotation = Rotation { x: -1, y: 1 };
static ROTATE_LEFT: Rotation = Rotation { x: 1, y: -1 };


fn main() -> Result<()> {
    Lazy::force(&RE);

    let instructions = parse()?;

    let mut min_x = 0;
    let mut min_y = 0;
    let mut max_x = 0;
    let mut max_y = 0;
    let mut current = Coord {
        x: 0,
        y: 0,
    };

    for instruction in &instructions {
        current = current.offset(instruction.direction * instruction.count as i64);
        if current.x > max_x {
            max_x = current.x
        }
        if current.y > max_y {
            max_y = current.y
        }
        if current.x < min_x {
            min_x = current.x
        }
        if current.y < min_y {
            min_y = current.y
        }
    }

    let rows = max_y - min_y + 1;
    let columns = max_x - min_x + 1;

    let mut map = Map {
        points: (0..rows * columns).map(|_| Point::Ground).collect(),
        rows: rows as usize,
        columns: columns as usize,
    };

    let mut current = Coord {
        x: -min_x,
        y: -min_y,
    };

    let mut rightness: i64 = 0;
    let mut previous: Option<Offset> = None;
    let mut border = HashSet::<Coord>::new();

    for instruction in &instructions {
        for _ in 0..instruction.count {
            *map.index_coord_mut(&current) = Point::Dig;
            border.insert(current);
            current = current.offset(instruction.direction);
        }
        if let Some(previous) = previous {
            rightness += cross_product(previous, instruction.direction) as i64;
        }

        previous = Some(instruction.direction);
    }

    let rotate_to_inside = match rightness.cmp(&0) {
        Ordering::Less => ROTATE_LEFT,
        Ordering::Greater => ROTATE_RIGHT,
        Ordering::Equal => bail!("unexpectedly straight loop"),
    };

    let mut current = Coord {
        x: -min_x,
        y: -min_y,
    };

    for instruction in &instructions {
        for _ in 0..instruction.count {
            let previous = current;
            current = current.offset(instruction.direction);

            let to_inside_offset = rotate(instruction.direction, rotate_to_inside);
            for from_coord in [previous, current] {
                let mut checked_coord = from_coord;
                loop {
                    checked_coord = checked_coord.offset(to_inside_offset);
                    if !map.is_valid(&checked_coord) {
                        break;
                    }
                    let checked_point = map.index_coord_mut(&checked_coord);
                    if let Point::Ground = checked_point {
                        *checked_point = Point::Dig;
                    }

                    if border.get(&checked_coord).is_some() {
                        break;
                    }
                }
            }
        }
    }

    println!("{}", map.points.iter().filter(|x| matches!(x, Point::Dig)).count());

    Ok(())
}

fn cross_product(a: Offset, b: Offset) -> i8 {
    ((a.x * b.y) - (a.y * b.x)) as i8
}

fn rotate(a: Offset, rotation: Rotation) -> Offset {
    Offset {
        x: a.y * rotation.x as i64,
        y: a.x * rotation.y as i64,
    }
}


struct Input {
    direction: Offset,
    count: u8,
}

static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\w) (\w+) \(#(\w{6})\)").unwrap());

fn parse() -> Result<Vec<Input>> {
    let stdin = io::stdin();
    let mut result = Vec::<Input>::new();
    for line in stdin.lines() {
        let line = line?;
        let captures = RE.captures(line.as_str()).context("invalid line")?;
        let (_, groups) = captures.extract::<3>();
        result.push(Input {
            direction: match groups[0] {
                "R" => RIGHT,
                "D" => DOWN,
                "L" => LEFT,
                "U" => UP,
                _ => bail!("invalid direction")
            },
            count: groups[1].parse()?,
        })
    }


    Ok(result)
}