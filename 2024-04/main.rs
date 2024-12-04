use anyhow::{bail, Result};
use std::io::stdin;
use std::ops::{Add, Index};

struct Grid {
    points: Vec<Point>,
    rows: usize,
    columns: usize,
}

impl Grid {
    fn index(&self, x: usize, y: usize) -> &Point {
        self.points.index(y * self.columns + x)
    }

    fn index_coord(&self, coord: Coord) -> &Point {
        self.index(coord.x as usize, coord.y as usize)
    }

    fn is_valid(&self, coord: Coord) -> bool {
        coord.x >= 0 && coord.y >= 0 && coord.x < self.columns as i64 && coord.y < self.rows as i64
    }

    fn iter(&self) -> impl Iterator<Item = (Coord, &Point)> {
        GridIterator{
            grid: self,
            next: if self.rows > 0 && self.columns > 0 {
                Some(Coord::default())
            } else {
                None
            },
        }
    }

    fn iter_line(&self, from: Coord, direction: Offset) -> impl Iterator<Item = (Coord, &Point)> {
        GridLineIterator{
            grid: self,
            next: Some(from),
            direction,
        }
    }
}

struct GridIterator<'a> {
    grid: &'a Grid,
    next: Option<Coord>,
}

impl<'a> Iterator for GridIterator<'a> {
    type Item = (Coord, &'a Point);

    fn next(&mut self) -> Option<Self::Item> {
        let res = self.next;
        if let Some(next) = &mut self.next {
            next.x += 1;
            if next.x >= self.grid.columns as i64 {
                next.x = 0;
                next.y += 1;
            }
            if next.y >= self.grid.rows as i64 {
                self.next = None;
            }
        }
        res.map(|c| (c, self.grid.index_coord(c)))
    }
}

struct GridLineIterator<'a> {
    grid: &'a Grid,
    next: Option<Coord>,
    direction: Offset,
}

impl Add<Offset> for Coord {
    type Output = Coord;

    fn add(self, rhs: Offset) -> Self::Output {
        Coord {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<'a> Iterator for GridLineIterator<'a> {
    type Item = (Coord, &'a Point);

    fn next(&mut self) -> Option<Self::Item> {
        let res = self.next;
        if let Some(next) = self.next.as_mut() {
            *next = *next + self.direction;
            if !self.grid.is_valid(*next) {
                self.next = None
            }
        }
        res.map(|c| (c, self.grid.index_coord(c)))
    }
}

#[derive(Eq, PartialEq, Debug, Copy, Clone, Hash, Default)]
struct Coord {
    x: i64,
    y: i64,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
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

    fn rotate_left(self) -> Offset {
        self.rotate(ROTATE_LEFT)
    }

    fn rotate_right(self) -> Offset {
        self.rotate(ROTATE_RIGHT)
    }
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

type Point = char;

const DIRECTIONS_8: [Offset; 8] = [
    Offset{x: 1, y: 0},
    Offset{x: 1, y: 1},
    Offset{x: 0, y: 1},
    Offset{x: -1, y: 1},
    Offset{x: -1, y: 0},
    Offset{x: -1, y: -1},
    Offset{x: 0, y: -1},
    Offset{x: 1, y: -1},
];

const DIRECTIONS_X: [Offset; 4] = [
    Offset{x: 1, y: 1},
    Offset{x: -1, y: 1},
    Offset{x: -1, y: -1},
    Offset{x: 1, y: -1},
];

// simplified  2d matrix, only for 90 degrees rotation
#[derive(Copy, Clone)]
struct Rotation {
    x: i8,
    y: i8,
}

static ROTATE_RIGHT: Rotation = Rotation { x: -1, y: 1 };
static ROTATE_LEFT: Rotation = Rotation { x: 1, y: -1 };

fn main() -> Result<()> {
    let grid = parse_input()?;

    println!("{}", part2(&grid));

    Ok(())
}

fn part1(grid: &Grid) -> i32 {
    let mut count = 0;
    for (coord, point) in grid.iter() {
        const TARGET: &str = "XMAS";

        if *point != TARGET.chars().next().unwrap() {
            continue
        }
        for direction in DIRECTIONS_8 {
            if grid
                .iter_line(coord, direction)
                .map(|(_, &p)| p)
                .skip(1)
                .take(TARGET.len() - 1)
                .eq(TARGET[1..].chars()) {
                count += 1
            }
        }
    }
    count
}

fn part2(grid: &Grid) -> i32 {
    let mut count = 0;
    for (coord, point) in grid.iter() {
        const TARGET: &str = "MAS";

        if *point != TARGET.chars().next().unwrap() {
            continue
        }
        for direction in DIRECTIONS_X {
            if grid
                .iter_line(coord, direction)
                .map(|(_, &p)| p)
                .skip(1)
                .take(TARGET.len() - 1)
                .eq(TARGET[1..].chars()) {
                let mid_coord = coord + direction;
                let left_coord = mid_coord + direction.rotate_left();
                let right_coord = mid_coord + direction.rotate_right();
                if grid.is_valid(left_coord)
                    && grid.is_valid(right_coord)
                    && (
                    (*grid.index_coord(left_coord) == 'M' && *grid.index_coord(right_coord) == 'S')
                    // Checking for both orientation of the cross would actually double count the crosses.
                    // By assuming arbitrary but consistent direction, only one of the two will be counted.
                    // || (*grid.index_coord(left_coord) == 'S' && *grid.index_coord(right_coord) == 'M')
                )

                {
                    count += 1
                }
            }
        }
    }
    count
}

fn parse_input() -> Result<Grid> {
    let mut result = Grid{
        points: vec![],
        rows: 0,
        columns: 0,
    };

    for line in stdin().lines() {
        let line = line?;
        if result.columns != 0 {
            if line.len() != result.columns {
                bail!("inconsistent line lengths");
            }
        } else {
            result.columns = line.len()
        }
        result.rows += 1;
        result.points.extend(line.chars())
    }

    Ok(result)
}