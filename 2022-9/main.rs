use std::collections::{HashSet};
use std::{io, iter};
use anyhow::{Context, Error};

#[derive(Copy, Clone, Default, PartialEq, Eq, Hash)]
struct Coord {
    x: i32,
    y: i32,
}

#[derive(Copy, Clone, Default)]
struct Offset {
    x: i32,
    y: i32,
}

#[derive(Default)]
struct RopePart {
    coord: Coord,
}

struct ParsedLine {
    offset: Offset,
    amount: i32,
}

struct Rope {
    parts: Vec<RopePart>
}

impl Rope {
    fn new(knots_including_head: u32) -> Rope {
        Rope{
            parts: iter::repeat_with(Default::default).take(knots_including_head as usize).collect(),
        }
    }

    fn move_head_by(&mut self, offset: Offset) {
        let (mut lead_part, tail ) = self.parts.split_first_mut().unwrap();
        lead_part.move_by(offset);
        for tail_part in tail  {
            tail_part.move_towards(lead_part);
            lead_part = tail_part;
        }
    }
}

impl RopePart {
    fn move_by(&mut self, offset: Offset) {
        self.coord.x += offset.x;
        self.coord.y += offset.y;
    }

    fn get_offset_to(&self, target: &RopePart) -> Offset {
        Offset {
            x: target.coord.x - self.coord.x,
            y: target.coord.y - self.coord.y,
        }
    }

    fn move_towards(&mut self, target: &RopePart) {
        let offset = self.get_offset_to(target);
        if !(offset.x.abs() > 1 || offset.y.abs() > 1) {
            return;
        }

        let offset = Offset {
            x: offset.x.clamp(-1, 1),
            y: offset.y.clamp(-1, 1),
        };

        self.move_by(offset);
    }
}

fn main() -> Result<(), Error> {
    // part 1
    // let mut rope = Rope::new(2);
    // part 2
    let mut rope = Rope::new(10);

    let mut tail_visited = HashSet::<Coord>::new();
    tail_visited.insert(rope.parts.last().unwrap().coord);

    let stdin = io::stdin();
    for line in stdin.lines() {
        let line = line?;

        let parsed_line = parse_line(&line)?;

        for _ in 0..parsed_line.amount {
            rope.move_head_by(parsed_line.offset);

            tail_visited.insert(rope.parts.last().unwrap().coord);
        }
    }

    println!("{}", tail_visited.len());

    Ok(())
}

fn parse_line(line: &str) -> Result<ParsedLine, Error> {
    let (direction_str, amount_str) = line.split_once(" ").context("invalid line")?;
    Ok(ParsedLine {
        offset: match direction_str {
            "R" => Offset {
                x: 1,
                y: 0,
            },
            "U" => Offset {
                x: 0,
                y: 1,
            },
            "L" => Offset {
                x: -1,
                y: 0,
            },
            "D" => Offset {
                x: 0,
                y: -1,
            },
            _ => {
                return Err(Error::msg("unrecognized direction"));
            }
        },
        amount: amount_str.parse::<i32>().context("parasing move amount")?,
    })
}