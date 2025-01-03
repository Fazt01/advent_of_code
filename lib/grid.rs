use std::convert::Infallible;
use std::iter;
use std::ops::{Add, Index, IndexMut, Neg, Sub};
use thiserror::Error;

pub struct Grid<T> {
    points: Vec<T>,
    rows: usize,
    columns: usize,
}

#[derive(Error, Debug)]
pub enum FromLinesTryIterMapError<E1, E2> {
    #[error("iterating line {1}: {0}")]
    LinesIteration(E1, usize),
    #[error("iterating line {1} item {2}: {0}")]
    LineIteration(E2, usize, usize),
    #[error("inconsistent line {0} length")]
    InconsistentLineLengths(usize),
}

type FromLinesIter = FromLinesTryIterMapError<Infallible, Infallible>;
type FromLinesTryIter<E1, E2> = FromLinesTryIterMapError<E1, E2>;

impl<T> Grid<T> {
    pub fn from_lines_iter<
        ILines: IntoIterator<Item = ILine>,
        ILine: IntoIterator<Item = T>,
    > (
        iter: ILines,
    ) -> Result<Self, FromLinesIter> {
        Self::from_lines_iter_map(iter, |_, x| x)
    }

    pub fn from_lines_iter_map<
        ILines: IntoIterator<Item = ILine>,
        ILine: IntoIterator<Item = U>,
        F: FnMut(Coord, U) -> T,
        U,
    > (
        iter: ILines,
        mut func: F,
    ) -> Result<Self, FromLinesIter> {
        Self::from_lines_try_iter_map(
            iter
                .into_iter()
                .map(|line| -> Result<_, Infallible> {
                    Ok(line.into_iter())
                }),
            |coord, item| -> Result<T, Infallible> {Ok(func(coord, item))}
        )
    }

    pub fn from_lines_try_iter<
        ILines: IntoIterator<Item = Result<ILine, E1>>,
        ILine: IntoIterator<Item = Result<T, E2>>,
        E1,
        E2,
    > (
        iter: ILines
    ) -> Result<Self, FromLinesTryIter<E1, E2>> {
        Self::from_lines_try_iter_map(iter, |_, x| x)
    }

    pub fn from_lines_try_iter_map<
        ILines: IntoIterator<Item = Result<ILine, E1>>,
        ILine: IntoIterator<Item = U>,
        F: FnMut(Coord, U) -> Result<T, E2>,
        U,
        E1,
        E2,
    > (
        iter: ILines,
        mut func: F,
    ) -> Result<Self, FromLinesTryIterMapError<E1, E2>> {
        let mut result = Grid{
            points: vec![],
            rows: 0,
            columns: 0,
        };

        for (y, line) in iter.into_iter().enumerate() {
            let line = line.map_err(|err|
                FromLinesTryIterMapError::LinesIteration(err, y)
            )?;
            let mut count = 0;
            for (x, item) in line.into_iter().enumerate() {
                let f_result = func(Coord{ x: x as i64, y: y as i64 }, item);
                result.points.push(f_result
                    .map_err(|err|
                        FromLinesTryIterMapError::LineIteration(err, y, x)
                    )?
                );
                count += 1;
            }
            if result.columns != 0 {
                if count != result.columns {
                    return Err(FromLinesTryIterMapError::InconsistentLineLengths(y));
                }
            } else {
                result.columns = count
            }
            result.rows += 1;
        }

        Ok(result)
    }

    pub fn new_sized_as<U>(src: &Grid<U>) -> Grid<T>
    where T: Default + Clone {
        Grid::<T>{
            points: iter::repeat(T::default()).take(src.columns * src.rows).collect(),
            rows: src.rows,
            columns: src.columns,
        }
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn columns(&self) -> usize {
        self.columns
    }

    pub fn index(&self, x: usize, y: usize) -> &T {
        self.points.index(y * self.columns + x)
    }

    pub fn index_mut(&mut self, x: usize, y: usize) -> &mut T {
        self.points.index_mut(y * self.columns + x)
    }

    pub fn index_coord(&self, coord: Coord) -> &T {
        self.index(coord.x as usize, coord.y as usize)
    }

    pub fn is_valid(&self, coord: Coord) -> bool {
        coord.x >= 0 && coord.y >= 0 && coord.x < self.columns as i64 && coord.y < self.rows as i64
    }

    pub fn get(&self,  coord: Coord) -> Option<&T> {
        if self.is_valid(coord) {
            Some(self.index(coord.x as usize, coord.y as usize))
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, coord: Coord) -> Option<&mut T> {
        if self.is_valid(coord) {
            Some(self.index_mut(coord.x as usize, coord.y as usize))
        } else {
            None
        }
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

    pub fn swap(&mut self, a: Coord, b: Coord) {
        let a = self.coord_to_index(a);
        let b = self.coord_to_index(b);
        self.points.swap(a, b)
    }

    fn coord_to_index(&self, coord: Coord) -> usize {
        coord.y as usize * self.columns + coord.x as usize
    }
}

impl<T: Default + Clone> Grid<T>{
    pub fn new(columns: usize, rows: usize) -> Grid<T>
    {
        Self::new_with_values(columns, rows, T::default())
    }
}

impl<T: Clone> Grid<T>{
    pub fn new_with_values(columns: usize, rows: usize, value: T) -> Grid<T>
    {
        Grid::<T>{
            points: iter::repeat(value).take(columns * rows).collect(),
            rows,
            columns,
        }
    }
}

impl<T> Index<Coord> for Grid<T> {
    type Output = T;

    fn index(&self, coord: Coord) -> &Self::Output {
        self.index(coord.x as usize, coord.y as usize)
    }
}

impl<T> IndexMut<Coord> for Grid<T> {
    fn index_mut(&mut self, coord: Coord) -> &mut Self::Output {
        self.index_mut(coord.x as usize, coord.y as usize)
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

impl Sub<Offset> for Coord {
    type Output = Coord;

    fn sub(self, rhs: Offset) -> Self::Output {
        Coord {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Sub for Coord {
    type Output = Offset;

    fn sub(self, rhs: Coord) -> Self::Output {
        Offset {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
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

#[derive(Eq, PartialEq, Debug, Copy, Clone, Hash, Default, Ord, PartialOrd)]
pub struct Coord {
    pub x: i64,
    pub y: i64,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default, Ord, PartialOrd)]
pub struct Offset {
    pub x: i64,
    pub y: i64,
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

impl Neg for Offset {
    type Output = Offset;

    fn neg(self) -> Self::Output {
        Offset {
            x: -self.x,
            y: -self.y,
        }
    }
}

pub const OFFSET_RIGHT: Offset = Offset{ x: 1, y: 0 };
pub const OFFSET_DOWN: Offset = Offset{ x: 0, y: 1 };
pub const OFFSET_LEFT: Offset = Offset{ x: -1, y: 0 };
pub const OFFSET_UP: Offset = Offset{ x: 0, y: -1 };


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

pub const DIRECTIONS_CARDINAL: [Offset; 4] = [
    Offset{x: 1, y: 0},
    Offset{x: 0, y: 1},
    Offset{x: -1, y: 0},
    Offset{x: 0, y: -1},
];

// simplified  2d matrix, only for 90 degrees rotation
#[derive(Copy, Clone)]
struct Rotation {
    x: i8,
    y: i8,
}

const ROTATE_RIGHT: Rotation = Rotation { x: -1, y: 1 };
const ROTATE_LEFT: Rotation = Rotation { x: 1, y: -1 };
