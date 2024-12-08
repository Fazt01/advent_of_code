use anyhow::Result;
use grid::{Coord, Grid, Offset};
use owned_chars::OwnedCharsExt;
use std::collections::HashMap;
use std::io::stdin;


fn main() -> Result<()> {
    let grid = parse_input()?;

    let antennas = group_antennas(&grid);
    let antinode_count = count_antinodes(&grid, &antennas);
    println!("{}", antinode_count);

    Ok(())
}

fn count_antinodes(grid: &Grid<char>, antennas: &HashMap<char, Vec<Coord>>) -> i32 {
    let mut antinodes = Grid::<bool>::new_sized_as(grid);
    for antennas in antennas.values() {
        for (i, &lhs) in antennas[..antennas.len() - 1].iter().enumerate() {
            for &rhs in &antennas[i+1..] {
                let offset = rhs - lhs;
                for (candidate, _) in iter_antinodes_part2(grid, lhs, rhs, offset)
                {
                    antinodes[candidate] = true;
                }
            }
        }
    }
    antinodes
        .iter()
        .filter(|(_, &has_antinode)| has_antinode)
        .count() as i32
}

fn iter_antinodes_part1(grid: &Grid<char>, lhs: Coord, rhs: Coord, offset: Offset) -> impl Iterator<Item = (Coord, &char)> {
    grid.iter_line(lhs, -offset).skip(1).take(1)
        .chain(grid.iter_line(rhs, offset).skip(1).take(1))
}

fn iter_antinodes_part2(grid: &Grid<char>, lhs: Coord, rhs: Coord, offset: Offset) -> impl Iterator<Item = (Coord, &char)> {
    grid.iter_line(lhs, -offset)
        .chain(grid.iter_line(rhs, offset))
}

fn group_antennas(grid: &Grid<char>) -> HashMap<char, Vec<Coord>> {
    let mut result: HashMap<char, Vec<Coord>> = HashMap::default();
    for (coord, &point) in grid.iter() {
        if point == '.' {
            continue
        }
        result.entry(point).or_default().push(coord);
    }

    result
}

fn parse_input() -> Result<Grid<char>> {
    let grid = itertools::process_results(
        stdin().lines().map(
            |line| -> Result<_> {
                Ok(line.map(|line| {
                    line.into_chars()
                })?)
            }
        ),
        |line| {
            Grid::from_lines_iter(line)
        },
    )??;

    Ok(grid)
}