use anyhow::{bail, Context, Result};
use itertools::Itertools;
use std::cmp::Reverse;
use std::collections::HashSet;
use std::io::stdin;

#[derive(Debug)]
struct Point([i64; 3]);

impl Point {
    fn distance(&self, other: &Self) -> i64 {
        (self.0[0].abs_diff(other.0[0]).pow(2)
            + self.0[1].abs_diff(other.0[1]).pow(2)
            + self.0[2].abs_diff(other.0[2]).pow(2)) as i64
    }
}

fn main() -> Result<()> {
    let points = parse_input()?;

    let mut distances = Vec::with_capacity(points.len() * (points.len() - 1) / 2);

    for i in 0..points.len() {
        for j in i + 1..points.len() {
            distances.push((i, j));
        }
    }

    distances.sort_by_key(|&(i, j)| points[i].distance(&points[j]));

    let mut circuits = Vec::from_iter((0..points.len()).map(|i| Some(HashSet::from([i]))));

    let mut circuits_map = Vec::from_iter(0..points.len());

    let mut merges_remaining = 1000;

    let mut last_connected_i = (0, 0);

    for (i, j) in distances {
        if merges_remaining == 0 {
            let mut circuits = circuits.iter().flatten().collect_vec();
            circuits.sort_by_key(|c| Reverse(c.len()));

            println!(
                "{}",
                circuits[0].len() * circuits[1].len() * circuits[2].len()
            );
        }
        merges_remaining -= 1;
        if circuits[circuits_map[i]]
            .as_ref()
            .context("primary circuit is already unused")?
            .contains(&j)
        {
            continue;
        }
        last_connected_i = (i, j);
        let other_circuit_i = circuits_map[j];
        let absorbed_set = circuits[other_circuit_i]
            .take()
            .context("expected still-used set, got None")?;
        for other_point_i in absorbed_set {
            circuits[circuits_map[i]]
                .as_mut()
                .context("inserting into empty unused circuit")?
                .insert(other_point_i);
            circuits_map[other_point_i] = circuits_map[i];
        }
    }

    println!(
        "{}",
        points[last_connected_i.0].0[0] * points[last_connected_i.1].0[0]
    );

    Ok(())
}

fn parse_input() -> Result<Vec<Point>> {
    Ok(stdin()
        .lines()
        .map(|line| {
            let line = line?;
            let parsed_vec: Vec<i64> = line
                .split(",")
                .map(|s| s.parse().with_context(|| format!("parsing '{}'", s)))
                .try_collect()?;
            if parsed_vec.len() != 3 {
                bail!("expected 3 coordinates, got '{}'", parsed_vec.len());
            }
            Ok(Point([parsed_vec[0], parsed_vec[1], parsed_vec[2]]))
        })
        .try_collect()?)
}
