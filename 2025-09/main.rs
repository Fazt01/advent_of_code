use anyhow::{bail, Context, Result};
use itertools::Itertools;
use lib::grid::Coord;
use std::cmp::{max, min};
use std::collections::HashMap;
use std::io::stdin;

fn main() -> Result<()> {
    let pole_coords = parse_input()?;

    let mut max_part1 = 0;

    let mut rect_viable: HashMap<(Coord, Coord), bool> = HashMap::default();

    for i in 0..pole_coords.len() - 1 {
        for j in i + 1..pole_coords.len() {
            rect_viable.insert((pole_coords[i], pole_coords[j]), true);

            let width = pole_coords[i].x.abs_diff(pole_coords[j].x) + 1;
            let height = pole_coords[i].y.abs_diff(pole_coords[j].y) + 1;
            let area = width * height;
            if area > max_part1 {
                max_part1 = area;
            }
        }
    }

    println!("{}", max_part1);

    for boundary_line in pole_coords
        .iter()
        .cloned()
        .chain([pole_coords[0]])
        .tuple_windows()
    {
        for (rect, viable) in rect_viable.iter_mut() {
            if crosses_boundary(boundary_line, *rect) {
                *viable = false
            }
        }
    }

    let mut max_part2 = 0;

    for (rect, viable) in &rect_viable {
        if !viable {
            continue;
        }

        let width = rect.0.x.abs_diff(rect.1.x) + 1;
        let height = rect.0.y.abs_diff(rect.1.y) + 1;
        let area = width * height;
        if area > max_part2 {
            max_part2 = area;
        }
    }

    println!("{}", max_part2);

    Ok(())
}

fn crosses_boundary(boundary_line: (Coord, Coord), rect: (Coord, Coord)) -> bool {
    let (from, to) = boundary_line;
    if from.x == to.x {
        if from.x <= min(rect.0.x, rect.1.x) || from.x >= max(rect.0.x, rect.1.x) {
            return false;
        }
        if max(from.y, to.y) <= min(rect.0.y, rect.1.y)
            || min(from.y, to.y) >= max(rect.0.y, rect.1.y)
        {
            return false;
        }
        true
    } else {
        if from.y <= min(rect.0.y, rect.1.y) || from.y >= max(rect.0.y, rect.1.y) {
            return false;
        }
        if max(from.x, to.x) <= min(rect.0.x, rect.1.x)
            || min(from.x, to.x) >= max(rect.0.x, rect.1.x)
        {
            return false;
        }
        true
    }
}

fn parse_input() -> Result<Vec<Coord>> {
    Ok(stdin()
        .lines()
        .map(|line| {
            let line = line?;
            let parsed_vec: Vec<i64> = line
                .split(",")
                .map(|s| s.parse().with_context(|| format!("parsing '{}'", s)))
                .try_collect()?;
            if parsed_vec.len() != 2 {
                bail!("expected 2 coordinates, got '{}'", parsed_vec.len());
            }
            Ok(Coord {
                x: parsed_vec[0],
                y: parsed_vec[1],
            })
        })
        .try_collect()?)
}

#[cfg(test)]
mod tests {
    use crate::crosses_boundary;
    use lib::grid::Coord;
    use rstest::rstest;

    #[rstest]
    #[case(
        (Coord { x: 2, y: 3 }, Coord { x: 7, y: 3 }),
        (Coord { x: 11, y: 1 }, Coord { x: 2, y: 5 }),
        true)
    ]
    #[case(
        (Coord { x: 0, y: 3 }, Coord { x: 2, y: 3 }),
        (Coord { x: 11, y: 1 }, Coord { x: 2, y: 5 }),
        false)
    ]
    #[case(
        (Coord { x: 2, y: 1 }, Coord { x: 7, y: 1 }),
        (Coord { x: 11, y: 1 }, Coord { x: 2, y: 5 }),
        false)
    ]
    #[case(
        (Coord { x: 2, y: 3 }, Coord { x: 15, y: 3 }),
        (Coord { x: 11, y: 1 }, Coord { x: 2, y: 5 }),
        true)
    ]
    fn test_crosses_boundary(
        #[case] line: (Coord, Coord),
        #[case] rect: (Coord, Coord),
        #[case] expected: bool,
    ) {
        assert_eq!(crosses_boundary(line, rect), expected)
    }
}
