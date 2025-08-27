use anyhow::Result;
use lib::grid::{Coord, Grid, DIRECTIONS_CARDINAL, OFFSET_DOWN, OFFSET_LEFT, OFFSET_RIGHT, OFFSET_UP};
use owned_chars::OwnedCharsExt;
use std::collections::{HashMap, HashSet};
use std::io::stdin;
use itertools::Itertools;

#[derive(Default)]
struct Region {
    slots: HashSet<Coord>,
    crop: char,
}

fn main() -> Result<()> {
    let input = parse_input()?;

    let regions = group_regions(&input);

    println!("{}", regions.iter().map(|r| discounted_region_price(r)).sum::<u64>());

    Ok(())
}

fn group_regions(grid: &Grid<char>) -> Vec<Region> {
    let mut slot_to_region: HashMap<Coord, usize> = Default::default();
    let mut regions: Vec<Region> = Default::default();

    for (coord, &crop) in grid.iter() {
        let mut my_regions: Vec<usize> = Default::default();
        for offset in [OFFSET_UP, OFFSET_LEFT] {
            let neighbor = coord + offset;
            if grid.is_valid(neighbor) {
                let neighbor_region_idx = slot_to_region[&neighbor];
                if regions[neighbor_region_idx].crop == crop {
                    my_regions.push(neighbor_region_idx)
                }
            }
        }
        match my_regions[..] {
            [] => {
                regions.push(Region{
                    slots: HashSet::from([coord]),
                    crop,
                });
                slot_to_region.insert(coord, regions.len()-1);
            }
            [x] => {
                regions[x].slots.insert(coord);
                slot_to_region.insert(coord, x);
            }
            [x, y] if x != y => {
                // merge two different regions that just met. Preserve x and destroy y.
                let x_ref;
                let y_ref;
                if x < y {
                    let (a,b) = regions.split_at_mut(y);
                    x_ref = &mut a[x];
                    y_ref = &mut b[0];
                } else {
                    let (a, b) = regions.split_at_mut(x);
                    y_ref = &mut a[y];
                    x_ref = &mut b[0];
                }
                for &destroyed_region_coord in &y_ref.slots {
                    slot_to_region.insert(destroyed_region_coord, x);
                    x_ref.slots.insert(destroyed_region_coord);
                }
                regions[x].slots.insert(coord);
                regions[y] = Default::default();
                slot_to_region.insert(coord, x);
            }
            [x, y] if x == y => {
                regions[x].slots.insert(coord);
                slot_to_region.insert(coord, x);
            }
            _ => unreachable!(),
        }
    }

    regions
}

fn region_price(region: &Region) -> u64 {
    let mut perimeter = 0_u64;
    for &slot in &region.slots {
        for offset in DIRECTIONS_CARDINAL {
            let neighbor = slot + offset;
            if !region.slots.contains(&neighbor) {
                perimeter += 1;
            }
        }
    }

    perimeter * (region.slots.len() as u64)
}

fn discounted_region_price(region: &Region) -> u64 {
    let mut perimeter = 0_u64;

    let mut sorted_x: Vec<Coord> = region.slots.iter().copied().collect();
    sorted_x.sort_by(|&a, &b| a.x.cmp(&b.x).then(a.y.cmp(&b.y)));
    let mut sorted_y: Vec<Coord> = region.slots.iter().copied().collect();
    sorted_y.sort_by(|&a, &b| a.y.cmp(&b.y).then(a.x.cmp(&b.x)));

    for offset in [OFFSET_LEFT, OFFSET_RIGHT] {
        let mut last_x = 0;
        let mut last_y = None;
        for &coord in &sorted_x {
            if last_x != coord.x {
                last_x = coord.x;
                last_y = None;
            }
            let neighbor = coord + offset;
            if !region.slots.contains(&neighbor) {
                if !matches!(last_y, Some(last_y_val) if last_y_val+1 == coord.y) {
                    perimeter += 1;
                }
                last_y = Some(coord.y);
            }
        }
    }

    for offset in [OFFSET_UP, OFFSET_DOWN] {
        let mut last_y = 0;
        let mut last_x = None;
        for &coord in &sorted_y {
            if last_y != coord.y {
                last_y = coord.y;
                last_x = None;
            }
            let neighbor = coord + offset;
            if !region.slots.contains(&neighbor) {
                if !matches!(last_x, Some(last_x_val) if last_x_val+1 == coord.x) {
                    perimeter += 1;
                }
                last_x = Some(coord.x);
            }
        }
    }

    perimeter * (region.slots.len() as u64)
}

fn parse_input() -> Result<Grid<char>> {
    Ok(Grid::from_lines_try_iter(stdin().lines().map(|line| -> Result<_> {
        Ok(line?.into_chars().map(|c| -> Result<_> { Ok(c) }))
    }))?)
}
