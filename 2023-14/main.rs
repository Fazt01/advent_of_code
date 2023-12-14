use std::io;
use std::ops::{Index, IndexMut, Range};
use anyhow::{Result, Ok, bail};

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

    fn index_mut(&mut self, x: usize, y: usize) -> &mut Point {
        self.points.index_mut(y * self.columns + x)
    }

    fn is_valid(&self, coord: &Coord) -> bool {
        coord.x >= 0 && coord.y >= 0 && coord.x < self.columns as i64 && coord.y < self.rows as i64
    }

    fn rock_load(&self, y: usize) -> u64 {
        (self.rows - y) as u64
    }

    fn rocks_load(&self) -> u64 {
        let mut sum = 0;
        for y in 0..self.rows {
            for x in 0..self.columns {
                match self.index(x,y) {
                    Point::RoundRock => {
                        sum += self.rock_load(y);
                    }
                    _ => {}
                }
            }
        }
        sum
    }

    fn roll(&mut self, roll_direction: Offset) {
        for y in self.rows_range(roll_direction) {
            for x in self.columns_range(roll_direction) {
                if !matches!(self.index(x,y), Point::RoundRock) {
                    continue
                }
                let roll_from = coord(x,y);
                let mut roll_to = roll_from;
                loop {
                    let maybe_roll_to = roll_to.offset(roll_direction);
                    if !self.is_valid(&maybe_roll_to) {
                        break
                    }
                    match self.index(maybe_roll_to.x as usize, maybe_roll_to.y as usize) {
                        Point::Empty => {
                            roll_to = maybe_roll_to;
                        }
                        _ => {
                            break
                        }
                    }
                }
                if roll_from != roll_to {
                    *self.index_mut(x,y) = Point::Empty;
                    *self.index_mut(roll_to.x as usize, roll_to.y as usize) = Point::RoundRock;
                }
            }
        }
    }
    fn rows_range(&self, roll_direction: Offset) -> Box<dyn Iterator<Item=usize>> {
        let result: Range<usize> = 0..self.rows;
        if roll_direction.y > 0 {
            return Box::from(result.rev())
        }
        Box::from(result)
    }
    fn columns_range(&self, roll_direction: Offset) -> Box<dyn Iterator<Item=usize>> {
        let result: Range<usize> = 0..self.columns;
        if roll_direction.x > 0 {
            return Box::from(result.rev())
        }
        Box::from(result)
    }
}

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
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

fn coord(x: usize, y: usize) -> Coord {
    Coord{
        x: x as i64,
        y: y as i64,
    }
}

#[derive(PartialEq, Copy, Clone)]
enum Point {
    Empty,
    RoundRock,
    CubeRock,
}

#[derive(Copy, Clone)]
struct Offset {
    x: i64,
    y: i64,
}


static UP: Offset = Offset { x: 0, y: -1 };
static LEFT: Offset = Offset { x: -1, y: 0 };
static DOWN: Offset = Offset { x: 0, y: 1 };
static RIGHT: Offset = Offset { x: 1, y: 0 };

fn main() -> Result<()> {
    // manually found the cycle length is 72, and e.g. cycle 1936 will have same result as cycle 1bil
    // ¯\_(ツ)_/¯
    // println!("{}", (1_000_000_000-1936)%72);
    // return Ok(());

    let mut map = parse()?;

    for cycle in 0..1_000_000_000 {
        let map_prev = map.clone();
        for roll_direction in [UP, LEFT, DOWN, RIGHT] {
            map.roll(roll_direction)
        }
        if map.points == map_prev.points {
            break;
        }
        println!("{}: {}", cycle+1, map.rocks_load());
    }

    println!("{}", map.rocks_load());

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
            points.push(match c {
                '.' => Point::Empty,
                'O' => Point::RoundRock,
                '#' => Point::CubeRock,
                _ => bail!("invalid point")
            });
        }
    }
    Ok(Map {
        points,
        rows,
        columns,
    })
}
