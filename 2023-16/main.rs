use std::collections::HashSet;
use std::io;
use std::ops::{Index, Neg};
use anyhow::{Result, Ok, Context};
use thiserror::Error;

#[derive(Clone)]
struct Map {
    points: Vec<Point>,
    rows: usize,
    columns: usize,
}

impl Map {
    fn index(&self, x: usize, y: usize) -> &Point {
        self.points.index(y * self.columns + x)
    }

    fn is_valid(&self, coord: &Coord) -> bool {
        coord.x >= 0 && coord.y >= 0 && coord.x < self.columns as i64 && coord.y < self.rows as i64
    }

    fn trace(&self, from: LightTrace) -> HashSet<LightTrace> {
        let mut result = HashSet::new();

        let mut input_rays = vec![from];

        while !input_rays.is_empty() {
            let input_ray = input_rays.pop().unwrap();
            let current_point = self.index(input_ray.coord.x as usize, input_ray.coord.y as usize);
            let output_directions = current_point.to_output_directions(input_ray.direction);
            for output_direction in output_directions {
                let output_ray = LightTrace {
                    coord: input_ray.coord,
                    direction: output_direction,
                };
                if !result.insert(output_ray) {
                    continue;
                }
                let new_input_ray = LightTrace {
                    coord: input_ray.coord.offset(output_direction),
                    direction: output_direction,
                };
                if !self.is_valid(&new_input_ray.coord) {
                    continue;
                }
                input_rays.push(new_input_ray)
            }
        }

        result
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

#[derive(Clone)]
enum Point {
    Empty,
    Mirror(i64),
    Splitter(Vec<Offset>),
}

#[derive(Error, Debug)]
#[error("invalid char")]
struct InvalidChar;

impl TryFrom<char> for Point {
    type Error = InvalidChar;

    fn try_from(value: char) -> std::result::Result<Self, Self::Error> {
        Result::Ok(match value {
            '.' => Point::Empty,
            '/' => Point::Mirror(-1),
            '\\' => Point::Mirror(1),
            '-' => Point::Splitter(vec![LEFT, RIGHT]),
            '|' => Point::Splitter(vec![UP, DOWN]),
            _ => return Err(InvalidChar {})
        })
    }
}

impl Point {
    fn to_output_directions(&self, input_direction: Offset) -> Vec<Offset> {
        match self {
            Point::Empty => vec![input_direction],
            Point::Mirror(mult) => vec![Offset {
                x: input_direction.y * mult,
                y: input_direction.x * mult,
            }],
            Point::Splitter(split_directions) => split_directions
                .iter()
                .filter(|x| **x != -input_direction)
                .map(|x| *x)
                .collect()
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
struct Offset {
    x: i64,
    y: i64,
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

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
struct LightTrace {
    coord: Coord,
    direction: Offset,
}


static UP: Offset = Offset { x: 0, y: -1 };
static LEFT: Offset = Offset { x: -1, y: 0 };
static DOWN: Offset = Offset { x: 0, y: 1 };
static RIGHT: Offset = Offset { x: 1, y: 0 };

fn main() -> Result<()> {
    let map = parse()?;

    let mut max = 0;

    for from_column in 0..map.columns {
        for from_direction in [DOWN, UP] {
            let from = LightTrace {
                coord: Coord { x: from_column as i64, y: if from_direction == DOWN { 0 } else { map.rows as i64 - 1 } },
                direction: from_direction,
            };
            let light_traces = map.trace(from);
            let unique_tiles = light_traces.iter().map(|x| x.coord).collect::<HashSet<_>>().len();
            max = std::cmp::max(max, unique_tiles)
        }
    }
    for from_row in 0..map.rows {
        for from_direction in [RIGHT, LEFT] {
            let from = LightTrace {
                coord: Coord { x: if from_direction == RIGHT { 0 } else { map.columns as i64 - 1 }, y: from_row as i64 },
                direction: from_direction,
            };
            let light_traces = map.trace(from);
            let unique_tiles = light_traces.iter().map(|x| x.coord).collect::<HashSet<_>>().len();
            max = std::cmp::max(max, unique_tiles)
        }
    }

    println!("{}", max);

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
            points.push(c.try_into().context("invalid char")?);
        }
    }
    Ok(Map {
        points,
        rows,
        columns,
    })
}


#[cfg(test)]
mod tests {
    use rstest::rstest;
    use crate::{UP, LEFT, RIGHT, DOWN, Point, Offset};

    #[rstest]
    #[case(RIGHT, '.', vec ! [RIGHT])]
    #[case(RIGHT, '/', vec ! [UP])]
    #[case(RIGHT, '\\', vec ! [DOWN])]
    #[case(RIGHT, '|', vec ! [UP, DOWN])]
    #[case(RIGHT, '-', vec ! [RIGHT])]
    #[case(DOWN, '-', vec ! [LEFT, RIGHT])]
    fn test_output_offsets(#[case] input: Offset, #[case] c: char, #[case] expected_output: Vec<Offset>) {
        let point: Point = c.try_into().unwrap();
        assert_eq!(point.to_output_directions(input), expected_output)
    }
}