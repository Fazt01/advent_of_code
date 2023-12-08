use std::collections::HashMap;
use std::io;
use std::ptr::NonNull;
use anyhow::{Result, Ok, Context, bail};
use once_cell::sync::Lazy;
use regex::Regex;

struct InputNode {
    name: String,
    left: String,
    right: String,
}

struct Puzzle {
    directions: Vec<Direction>,
    desert: DesertMap,
}

struct DesertMap {
    places: HashMap<String, Box<Place>>,
}

impl DesertMap {
    fn from_node_vec(input_nodes: Vec<InputNode>) -> Result<DesertMap> {
        let mut result = HashMap::<String, Box<Place>>::new();

        for node in &input_nodes {
            let name: String = (&node.name).clone();
            let name_ptr = NonNull::from(name.as_str());
            result.insert(name, Place {
                name: name_ptr,
                left: None,
                right: None,
            }.into());
        }

        for node in &input_nodes {
            let name = node.name.as_str();
            let left_name = node.left.as_str();
            let right_name = node.right.as_str();
            let left_place = Some(match result.get(left_name) {
                None => { bail!("unknow place  {}", left_name) }
                Some(left_place) => { (*left_place).as_ref().into() }
            });
            let right_place = Some(match result.get(left_name) {
                None => { bail!("unknow place  {}", right_name) }
                Some(right_place) => { (*right_place).as_ref().into() }
            });

            match result.get_mut(name) {
                None => { unreachable!("the name should have been inserted in loop above") }
                Some(place) => {
                    place.left = left_place;
                    place.right = right_place;
                }
            }
        }

        Ok(DesertMap {
            places: result,
        })
    }
}

struct Place {
    name: NonNull<str>,
    left: Option<NonNull<Place>>,
    right: Option<NonNull<Place>>,
}

enum Direction {
    Left,
    Right,
}

impl Direction {
    fn from_char(c: &char) -> Option<Direction> {
        match c {
            'L' => Some(Direction::Left),
            'R' => Some(Direction::Right),
            _ => None
        }
    }
}


static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"(\w+) = \((\w+), (\w+)\)"#).unwrap());

fn main() -> Result<()> {
    Lazy::force(&RE);

    let puzzle = parse()?;

    let mut current_place = puzzle.desert.places.get("AAA").context("missing starting place AAA")?.as_ref();
    let mut steps = 0;
    for direction in puzzle.directions.iter().cycle() {
        let go_to = match direction {
            Direction::Left => unsafe { current_place.left.context("uninitialized left place")?.as_ref() }
            Direction::Right => unsafe { current_place.right.context("uninitialized left place")?.as_ref() }
        };
        current_place = go_to;
        steps += 1;
        if unsafe {
            println!("{}", current_place.name.as_ref());
            current_place.name.as_ref()
        } == "ZZZ" {
            break;
        }
    }

    println!("{}", steps);

    Ok(())
}

fn parse() -> Result<Puzzle> {
    let stdin = io::stdin();
    let mut lines = stdin.lines();
    let first_line = lines.next();
    let directions: Vec<Direction> = first_line.context("no first line")??.chars().map(
        |c| Ok(Direction::from_char(&c).context("invalid direction")?)
    ).collect::<Result<_>>()?;
    lines.next(); // consume empty line

    let parsed_lines = lines.into_iter().map(|line| {
        let line = line?;
        let captures = RE.captures(&line).context("no match")?;
        let (_, groups) = captures.extract::<3>();
        Ok(InputNode {
            name: groups[0].to_string(),
            left: groups[1].to_string(),
            right: groups[2].to_string(),
        })
    }).collect::<Result<_>>()?;

    Ok(Puzzle {
        directions,
        desert: DesertMap::from_node_vec(parsed_lines)?,
    })
}