use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::{io};
use std::cmp::max;
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
            let right_place = Some(match result.get(right_name) {
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

    fn navigate(&self, current_place: &Place, direction: &Direction) -> Result<&Place> {
        Ok(match direction {
            Direction::Left => unsafe {
                current_place.left.context("uninitialized left place")?.as_ref()
            }
            Direction::Right => unsafe {
                current_place.right.context("uninitialized right place")?.as_ref()
            }
        })
    }
}

struct Place {
    name: NonNull<str>,
    left: Option<NonNull<Place>>,
    right: Option<NonNull<Place>>,
}

impl Debug for Place {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(unsafe { self.name.as_ref() })
    }
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

#[derive(Debug)]
enum CycleDetect {
    None,
    StartsAt(u64),
    CycleLength(u64),
}

static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"(\w+) = \((\w+), (\w+)\)"#).unwrap());

fn lcm(x: u64, y: u64) -> u64 {
    let x_primes = factors(x);
    let mut y_primes = factors(y);
    let mut result: u64 = 1;
    for (x_prime, x_count) in &x_primes {
        let y_count = y_primes.remove(x_prime).unwrap_or(0);
        result *= x_prime.pow(max(*x_count, y_count) as u32)
    }

    for (remaining_prime, remaining_count) in y_primes {
        result *= remaining_prime.pow(remaining_count as u32)
    }

    result
}

fn factors(x: u64) -> HashMap<u64, u64> {
    let mut x = x;
    let mut result = HashMap::new();

    'outer: loop {
        for factor in 2..=((x as f64).sqrt() as u64) {
            if x % factor == 0 {
                x = x / factor;
                result.insert(factor, *result.get(&factor).unwrap_or(&0) + 1);
                continue 'outer;
            }
        }
        break;
    }

    result.insert(x, *result.get(&x).unwrap_or(&0) + 1);

    result
}

fn main() -> Result<()> {
    println!("{}", 44810373917_u64 * 293_u64);

    Lazy::force(&RE);

    let puzzle = parse()?;

    // part 1
    let mut current_place = puzzle.desert.places.get("AAA").context("missing starting place AAA")?.as_ref();
    let mut steps: u64 = 0;
    for direction in puzzle.directions.iter().cycle() {
        current_place = puzzle.desert.navigate(current_place, &direction)?;
        steps += 1;
        if unsafe {
            current_place.name.as_ref()
        } == "ZZZ" {
            break;
        }
    }

    println!("{}", steps);

    // part 2
    let mut current_places: Vec<&Place> = puzzle.desert.places
        .iter()
        .filter(|(k, _)| k.ends_with("A"))
        .map(|(_, v)| (*v).as_ref())
        .collect();
    let mut cycle_detects = current_places
        .iter()
        .map(|_| CycleDetect::None)
        .collect::<Vec<_>>();
    let mut steps: u64 = 0;
    for direction in puzzle.directions.iter().cycle() {
        current_places = current_places
            .iter()
            .map(|current_place| Ok(puzzle.desert.navigate(current_place, &direction)?))
            .collect::<Result<_>>()?;
        steps += 1;

        for (i, current_place) in current_places.iter().enumerate() {
            if unsafe { current_place.name.as_ref().ends_with("Z") } {
                match cycle_detects[i] {
                    CycleDetect::None => {
                        cycle_detects[i] = CycleDetect::StartsAt(steps)
                    }
                    CycleDetect::StartsAt(starts_at) => {
                        cycle_detects[i] = CycleDetect::CycleLength(steps - starts_at)
                    }
                    CycleDetect::CycleLength(_) => {}
                }
            }
        }

        if cycle_detects
            .iter()
            .all(|place| matches!(place, CycleDetect::CycleLength(_))) {
            break;
        }
    }

    let mut least_common_multiple = 1;
    for cycle in &cycle_detects {
        match cycle {
            CycleDetect::CycleLength(cycle_len) => {
                least_common_multiple = lcm(least_common_multiple, *cycle_len)
            }
            _ => {}
        }
    }

    println!("{}", least_common_multiple);


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

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use std::collections::HashMap;
    use crate::{factors, lcm};

    #[rstest]
    #[case(99, HashMap::from_iter([(3, 2), (11, 1)]))]
    #[case(121, HashMap::from_iter([(11, 2)]))]
    #[case(128, HashMap::from_iter([(2, 7)]))]
    fn test_factors(#[case] x: u64, #[case] expected: HashMap<u64, u64>) {
        assert_eq!(factors(x), expected)
    }

    #[rstest]
    #[case(99, 121, 1089)]
    #[case(4, 5, 20)]
    #[case(128, 64, 128)]
    fn test_lcm(#[case] x: u64, #[case] y: u64,  #[case] expected: u64) {
        assert_eq!(lcm(x, y), expected)
    }
}