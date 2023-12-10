use std::collections::HashSet;
use std::io;
use std::ops::{Index, Neg};
use anyhow::{Result, Ok, bail, Context};

struct State {
    pipes: Vec<Vec<Pipe>>,
    start: Coord,
}

#[derive(Debug)]
enum Pipe {
    None,
    Simple([Offset; 2]),
    Start,
}

#[derive(Copy, Clone, PartialEq, Debug)]
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

struct Step {
    to: Coord,
    by: Offset,
    right_turn: i8, // -1 for left turn, 0 for straight, 1 for right turn
}

// simplified  2d matrix, only for 90 degrees rotation
#[derive(Copy, Clone)]
struct Rotation {
    x: i8,
    y: i8,
}

static ROTATE_RIGHT: Rotation = Rotation { x: -1, y: 1 };
static ROTATE_LEFT: Rotation = Rotation { x: 1, y: -1 };

fn main() -> Result<()> {
    let state = parse()?;

    let mut path: Vec<Step> = Vec::new();
    let mut visited: HashSet<Coord> = HashSet::new();
    let mut current_coord = state.start;
    let mut last_move: Option<Offset> = None;
    loop {
        let next = pipe_move(&state, current_coord, last_move)
            .with_context(|| format!("at: {:?}, from: {:?}", current_coord, last_move))?;
        current_coord = next.to;
        last_move = Some(next.by);
        visited.insert(next.to);
        path.push(next);
        if current_coord == state.start {
            break;
        }
    }

    let rightness: i64 = path.iter().map(|x| x.right_turn as i64).sum();
    let rotate_to_inside = if rightness < 0 {
        ROTATE_LEFT
    } else if rightness > 0 {
        ROTATE_RIGHT
    } else {
        bail!("unexpectedly straight loop");
    };

    let mut inner_coords: HashSet<Coord> = HashSet::new();
    for step in &path {
        let to_inside_offset = rotate(step.by, rotate_to_inside);
        // due to turns, check each step in path twice:
        // for "direction by which I got there" (from_coord = step.to)
        // and "direction I got from there" (from_coord = previous step coord, with current step.by)
        for from_coord in [step.to.offset(-step.by), step.to] {
            let mut checked_coord = from_coord;
            loop {
                checked_coord = checked_coord.offset(to_inside_offset);
                match visited.get(&checked_coord) {
                    None => {}
                    Some(_) => break
                }
                if checked_coord.x < 0 || checked_coord.y < 0
                    || checked_coord.x >= state.pipes[0].len() as i64 || checked_coord.y >= state.pipes.len() as i64 {
                    break;
                }
                inner_coords.insert(checked_coord);
            }
        }
    }

    println!("{}", path.len() / 2);

    println!("{}", inner_coords.len());

    Ok(())
}

fn pipe_move(state: &State, current_coord: Coord, last_move: Option<Offset>) -> Result<Step> {
    let current_pipe = state.pipes.index(current_coord.y as usize).index(current_coord.x as usize);
    match current_pipe {
        Pipe::None => bail!("currently in empty space"),
        Pipe::Simple(offsets) => {
            match last_move {
                None => bail!("don't know where I came from"),
                Some(from_offset) => {
                    let offset = next_offset(offsets, from_offset).context("couldn't get here from specified last_move offset")?;
                    let right_turn = cross_product(from_offset, offset);
                    let next_coord = current_coord.offset(offset);
                    Ok(Step { to: next_coord, by: offset, right_turn })
                }
            }
        }
        Pipe::Start => {
            for offset in [UP, LEFT, DOWN, RIGHT] {
                let potential_next_coord = current_coord.offset(offset);
                let potential_pipe = if potential_next_coord.x < 0 || potential_next_coord.y < 0 {
                    None
                } else {
                    state.pipes.get(potential_next_coord.y as usize)
                        .map(|row| row.get(potential_next_coord.x as usize))
                        .flatten()
                };
                match potential_pipe {
                    None => {} // this pipe would be out of bounds
                    Some(pipe) => {
                        if is_connected_from(pipe, offset) {
                            return Ok(Step { to: potential_next_coord, by: offset, right_turn: 0 });
                        }
                    }
                }
            }
            bail!("No escape from start found");
        }
    }
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

fn is_connected_from(pipe: &Pipe, offset: Offset) -> bool {
    let opposite_offset = Offset {
        x: -offset.x,
        y: -offset.y,
    };
    match pipe {
        Pipe::None => false,
        Pipe::Simple(offsets) => offsets.contains(&opposite_offset),
        Pipe::Start => false,
    }
}

fn next_offset(offsets: &[Offset; 2], offset: Offset) -> Option<Offset> {
    let opposite_offset = Offset {
        x: -offset.x,
        y: -offset.y,
    };
    let in_offset_index = offsets.iter().position(|x| *x == opposite_offset)?;
    Some(offsets[1 - in_offset_index])
}


fn parse() -> Result<State> {
    let stdin = io::stdin();
    let mut result: Vec<Vec<Pipe>> = Vec::new();
    let mut start: Option<Coord> = None;
    for (y, line) in stdin.lines().enumerate() {
        let line = line?;
        let mut line_vec: Vec<Pipe> = Vec::new();
        for (x, c) in line.chars().enumerate() {
            let pipe = match c {
                'S' => {
                    start = Some(Coord { x: x as i64, y: y as i64 });
                    Pipe::Start
                }
                '.' => Pipe::None,
                _ => Pipe::Simple(match c {
                    '-' => [LEFT, RIGHT],
                    '|' => [UP, DOWN],
                    'L' => [UP, RIGHT],
                    'F' => [RIGHT, DOWN],
                    'J' => [UP, LEFT],
                    '7' => [LEFT, DOWN],
                    _ => bail!("invalid pipe shape")
                })
            };
            line_vec.push(pipe);
        }
        result.push(line_vec)
    }

    Ok(State {
        pipes: result,
        start: start.context("no start pipe")?,
    })
}


#[cfg(test)]
mod tests {
    use rstest::rstest;
    use crate::{cross_product, rotate, Offset, Rotation, UP, LEFT, RIGHT, ROTATE_LEFT, ROTATE_RIGHT};

    #[rstest]
    #[case(UP, LEFT, - 1)]
    #[case(UP, RIGHT, 1)]
    #[case(UP, UP, 0)]
    fn test_cross_product(#[case] a: Offset, #[case] b: Offset, #[case] expected_right_turn: i8) {
        assert_eq!(cross_product(a, b), expected_right_turn)
    }

    #[rstest]
    #[case(UP, ROTATE_LEFT, LEFT)]
    #[case(UP, ROTATE_RIGHT, RIGHT)]
    fn test_rotate(#[case] a: Offset, #[case] rotation: Rotation, #[case] expected_offset: Offset) {
        assert_eq!(rotate(a, rotation), expected_offset)
    }
}