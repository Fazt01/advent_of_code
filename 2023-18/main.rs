use std::cmp::{max, min};
use std::io;
use std::ops::Mul;
use anyhow::{Result, Ok, bail, Context};
use once_cell::sync::Lazy;
use regex::Regex;

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

static UP: Offset = Offset { x: 0, y: -1 };
static LEFT: Offset = Offset { x: -1, y: 0 };
static DOWN: Offset = Offset { x: 0, y: 1 };
static RIGHT: Offset = Offset { x: 1, y: 0 };

#[derive(Copy, Clone, PartialEq, Debug, Hash, Eq, Default)]
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

struct BoundingBox {
    min_x: i64,
    min_y: i64,
    max_x: i64,
    max_y: i64,
}

impl BoundingBox {
    fn on_coord(coord: &Coord) -> BoundingBox {
        BoundingBox {
            min_x: coord.x,
            min_y: coord.y,
            max_x: coord.x,
            max_y: coord.y,
        }
    }
    fn include(&self, coord: &Coord) -> BoundingBox {
        BoundingBox {
            min_x: min(self.min_x, coord.x),
            min_y: min(self.min_y, coord.y),
            max_x: max(self.max_x, coord.x),
            max_y: max(self.max_y, coord.y),
        }
    }

    fn is_border(&self, coord: &Coord) -> bool {
        coord.x == self.min_x || coord.y == self.min_y || coord.x == self.max_x || coord.y == self.max_y
    }

    fn area(&self) -> i64 {
        (self.max_x - self.min_x) * (self.max_y - self.min_y)
    }
}

fn main() -> Result<()> {
    Lazy::force(&RE);

    let instructions = parse()?;

    let points = instructions_to_points(&instructions);

    let area = area_rec(&points, 0, points.len() - 1, 1);
    let perimeter_length = instructions
        .iter()
        .map(|x| x.count)
        .sum::<i64>();
    // adjustment for integer-grid - all areas are calculated "including top-left edges,
    // excluding bottom-right edges" to simplify adding and subtracting.
    // this final adjustments adds all bottom and right edges
    // (they have to be exactly half of perimeter, as border ends at the start).
    // +1 as single point has area of 1.
    let area_adjust = perimeter_length / 2 + 1;

    println!("{}", area + area_adjust);

    Ok(())
}

fn instructions_to_points(instructions: &Vec<Input>) -> Vec<Coord> {
    let mut result = Vec::<Coord>::new();
    let mut current = Coord::default();
    for instruction in instructions {
        result.push(current);
        current = current.offset(instruction.direction * instruction.count);
    }
    result
}

fn area_rec(points: &Vec<Coord>, start: usize, end: usize, multiplier: i64) -> i64 {
    let mut bounds: Option<BoundingBox> = None;
    let mut i = start;
    loop {
        let point = &points[i];
        bounds = Some(match bounds {
            None => BoundingBox::on_coord(point),
            Some(bounds) => bounds.include(point),
        });
        if i == end {
            break;
        }
        i = (i + 1) % points.len();
    }
    let bounds = bounds.unwrap();
    let own_area = multiplier * bounds.area();
    let mut sub_areas_sum = 0;

    let mut prev_on_border: Option<usize> = None;
    // initialize with 1 before start, so that `prev_on_border` is properly initialized for points[start]
    // (that is, prev_on_border.is_some() if points[start] is the first point to go off border)
    let mut i = (start + points.len() - 1) % points.len();
    let mut initializing = true;
    loop {
        let point = &points[i];
        if bounds.is_border(point) {
            prev_on_border = Some(i);
        } else {
            if let Some(prev_i) = prev_on_border {
                // find where this line attaches back to border
                let mut next_i = i;
                loop {
                    next_i = (next_i + 1) % points.len();
                    let next_point = &points[next_i];
                    if bounds.is_border(next_point) {
                        let sub_area = area_rec(points, prev_i, next_i, -multiplier);
                        sub_areas_sum += sub_area;
                        break;
                    }
                }
            }
            prev_on_border = None;
        }
        if i == end && !initializing{
            break;
        }
        i = (i + 1) % points.len();
        initializing = false;
    }

    own_area + sub_areas_sum
}

struct Input {
    direction: Offset,
    count: i64,
}

static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\w) (\w+) \(#(\w{6})\)").unwrap());

fn parse() -> Result<Vec<Input>> {
    let stdin = io::stdin();
    let mut result = Vec::<Input>::new();
    for line in stdin.lines() {
        let line = line?;
        let captures = RE.captures(line.as_str()).context("invalid line")?;
        let (_, groups) = captures.extract::<3>();
        let (hex_len, dir_digit) = groups[2].split_at(5);
        result.push(Input {
            // part 1
            // direction: match groups[0] {
            //     "R" => RIGHT,
            //     "D" => DOWN,
            //     "L" => LEFT,
            //     "U" => UP,
            //     _ => bail!("invalid direction")
            // },
            // count: groups[1].parse()?,

            // part 2
            direction: match dir_digit {
                "0" => RIGHT,
                "1" => DOWN,
                "2" => LEFT,
                "3" => UP,
                _ => bail!("invalid direction")
            },
            count: i64::from_str_radix(hex_len, 16)?,
        })
    }


    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::{area_rec, DOWN, Input, instructions_to_points, LEFT, RIGHT, UP};

    // Maybe there is some edge case when start is / is not on border.

    #[test]
    fn test_l_shape_indexing() {
        let mut input = vec![
            Input {
                direction: RIGHT,
                count: 10,
            },
            Input {
                direction: UP,
                count: 10,
            },
            Input {
                direction: RIGHT,
                count: 10,
            },
            Input {
                direction: DOWN,
                count: 20,
            },
            Input {
                direction: LEFT,
                count: 20,
            },
            Input {
                direction: UP,
                count: 10,
            },
        ];
        for i in 0..input.len() {
            let points = instructions_to_points(&input);
            let area = area_rec(&points, 0, points.len() - 1, 1);
            assert_eq!(area, 300, "rotation {}", i); // 400 main - 100 inner indent

            input.rotate_left(1);
        }
    }

    #[test]
    fn test_u_shape_indexing() {
        let mut input = vec![
            Input {
                direction: RIGHT,
                count: 10,
            },
            Input {
                direction: UP,
                count: 10,
            },
            Input {
                direction: LEFT,
                count: 10,
            },
            Input {
                direction: UP,
                count: 10,
            },
            Input {
                direction: RIGHT,
                count: 20,
            },
            Input {
                direction: DOWN,
                count: 30,
            },
            Input {
                direction: LEFT,
                count: 20,
            },
            Input {
                direction: UP,
                count: 10,
            },
        ];
        for i in 0..input.len() {
            let points = instructions_to_points(&input);
            let area = area_rec(&points, 0, points.len() - 1, 1);
            assert_eq!(area, 500, "rotation {}", i); // 600 main - 100 inner indent

            input.rotate_left(1);
        }
    }

    #[test]
    fn test_l_in_u_shape_indexing() {
        let mut input = vec![
            Input {
                direction: RIGHT,
                count: 10,
            },
            Input {
                direction: UP,
                count: 5,
            },
            Input {
                direction: LEFT,
                count: 5,
            },
            Input {
                direction: UP,
                count: 5,
            },
            Input {
                direction: LEFT,
                count: 5,
            },
            Input {
                direction: UP,
                count: 10,
            },
            Input {
                direction: RIGHT,
                count: 20,
            },
            Input {
                direction: DOWN,
                count: 30,
            },
            Input {
                direction: LEFT,
                count: 20,
            },
            Input {
                direction: UP,
                count: 10,
            },
        ];
        for i in 0..input.len() {
            let points = instructions_to_points(&input);
            let area = area_rec(&points, 0, points.len() - 1, 1);
            assert_eq!(area, 525, "rotation {}", i); // 600 main - 100 inner indent + 25 inner most outdent

            input.rotate_left(1);
        }
    }
}