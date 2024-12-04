use anyhow::{bail, Result};
use std::ops::{Add, Index};

pub struct Grid<T> {
    points: Vec<T>,
    rows: usize,
    columns: usize,
}

impl<T> Grid<T> {
    pub fn from_lines_iter<
        ILines: IntoIterator<Item =ILine>,
        ILine: IntoIterator<Item = T>,
    > (
        iter: ILines
    ) -> Result<Self> {
        let mut result = Grid{
            points: vec![],
            rows: 0,
            columns: 0,
        };

        for line in iter {
            let vec = Vec::<T>::from_iter(line);
            if result.columns != 0 {
                if vec.len() != result.columns {
                    bail!("inconsistent line lengths");
                }
            } else {
                result.columns = vec.len()
            }
            result.rows += 1;
            result.points.extend(vec)
        }

        Ok(result)
    }

    pub fn index(&self, x: usize, y: usize) -> &T {
        self.points.index(y * self.columns + x)
    }

    pub fn index_coord(&self, coord: Coord) -> &T {
        self.index(coord.x as usize, coord.y as usize)
    }

    pub fn is_valid(&self, coord: Coord) -> bool {
        coord.x >= 0 && coord.y >= 0 && coord.x < self.columns as i64 && coord.y < self.rows as i64
    }

    pub fn iter(&self) -> impl Iterator<Item = (Coord, &T)> {
        GridIterator{
            grid: self,
            next: if self.rows > 0 && self.columns > 0 {
                Some(Coord::default())
            } else {
                None
            },
        }
    }

    pub fn iter_line(&self, from: Coord, direction: Offset) -> impl Iterator<Item = (Coord, &T)> {
        GridLineIterator{
            grid: self,
            next: Some(from),
            direction,
        }
    }
}

struct GridIterator<'a, T> {
    grid: &'a Grid<T>,
    next: Option<Coord>,
}

impl<'a, T> Iterator for GridIterator<'a, T> {
    type Item = (Coord, &'a T);

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

struct GridLineIterator<'a, T> {
    grid: &'a Grid<T>,
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

impl<'a, T> Iterator for GridLineIterator<'a, T> {
    type Item = (Coord, &'a T);

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
pub struct Coord {
    x: i64,
    y: i64,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct Offset {
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

    pub fn rotate_left(self) -> Offset {
        self.rotate(ROTATE_LEFT)
    }

    pub fn rotate_right(self) -> Offset {
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

pub const DIRECTIONS_8: [Offset; 8] = [
    Offset{x: 1, y: 0},
    Offset{x: 1, y: 1},
    Offset{x: 0, y: 1},
    Offset{x: -1, y: 1},
    Offset{x: -1, y: 0},
    Offset{x: -1, y: -1},
    Offset{x: 0, y: -1},
    Offset{x: 1, y: -1},
];

pub const DIRECTIONS_X: [Offset; 4] = [
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

const ROTATE_RIGHT: Rotation = Rotation { x: -1, y: 1 };
const ROTATE_LEFT: Rotation = Rotation { x: 1, y: -1 };