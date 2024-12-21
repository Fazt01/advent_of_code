use anyhow::Result;
use grid::{Coord, Grid, Offset, OFFSET_DOWN, OFFSET_LEFT, OFFSET_RIGHT, OFFSET_UP};
use itertools::Itertools;
use std::collections::HashMap;
use std::io::stdin;

type Code = Vec<u8>;

struct Keyboard {
    grid: Grid<u8>,
    coords: HashMap<u8, Coord>,
}

impl From<Grid<u8>> for Keyboard {
    fn from(value: Grid<u8>) -> Self {
        let mut coords: HashMap<u8, Coord> = Default::default();

        for (coord, &button) in value.iter() {
            coords.insert(button, coord);
        }

        Keyboard {
            grid: value,
            coords,
        }
    }
}

const KEYBOARD_COUNT: u64 = 26;

fn main() -> Result<()> {
    let input = parse_input()?;

    let final_keypad: Keyboard = Grid::from_lines_iter(
        vec!["789", "456", "123", ".0A"]
            .iter()
            .map(|x| x.bytes().clone()),
    )?
    .into();
    let intermediate_keypad: Keyboard =
        Grid::from_lines_iter(vec![".^A", "<v>"].iter().map(|x| x.bytes().clone()))?.into();

    let mut cache: HashMap<Code, HashMap<u64, u64>> = Default::default();

    let mut sum = 0;
    for code in &input {
        let mut start_coord = final_keypad.coords[&b'A'];
        let mut len = 0;
        for &symbol in code {
            let dest_coord = final_keypad.coords[&symbol];
            let offset = dest_coord - start_coord;
            let paths = get_paths(offset);
            let mut candidate_codes = vec![];
            'paths: for path in &paths {
                let mut gap_test_coord = start_coord;
                for &offset in path {
                    gap_test_coord = gap_test_coord + offset;
                    if final_keypad.grid[gap_test_coord] == b'.' {
                        continue 'paths;
                    }
                }
                candidate_codes.push(symbols_with_activate(path))
            }
            start_coord = start_coord + offset;
            len += candidate_codes
                .iter()
                .map(|code| get_code_price(code, KEYBOARD_COUNT, &intermediate_keypad, &mut cache))
                .min()
                .unwrap()
        }

        let complexity = complexity(code, len)?;
        sum += complexity;
        println!("{complexity} {}", len);
        println!("{}", String::from_utf8(code.clone())?,);
    }
    println!("{sum}");

    Ok(())
}

fn get_code_price(
    code: &Vec<u8>,
    level: u64,
    keyboard: &Keyboard,
    mut cache: &mut HashMap<Code, HashMap<u64, u64>>,
) -> u64 {
    if let Some(cached) = cache.get(code) {
        if let Some(&cached_level) = cached.get(&level) {
            return cached_level;
        }
    }

    let mut len: u64 = 0;
    if level == 1 {
        len = code.len() as u64;
    } else {
        let mut start_coord = keyboard.coords[&b'A'];
        for &symbol in code {
            let dest_coord = keyboard.coords[&symbol];
            let offset = dest_coord - start_coord;
            let paths = get_paths(offset);
            let mut candidate_codes = vec![];
            'paths: for path in &paths {
                let mut gap_test_coord = start_coord;
                for &offset in path {
                    gap_test_coord = gap_test_coord + offset;
                    if keyboard.grid[gap_test_coord] == b'.' {
                        continue 'paths;
                    }
                }
                candidate_codes.push(symbols_with_activate(path))
            }
            start_coord = start_coord + offset;
            len += candidate_codes
                .iter()
                .map(|code| get_code_price(code, level - 1, &keyboard, &mut cache))
                .min()
                .unwrap()
        }
    }

    cache.entry(code.clone()).or_default().insert(level, len);

    len
}

fn get_paths(offset: Offset) -> Vec<Vec<Offset>> {
    let mut horizontal = Vec::new();
    let mut vertical = Vec::new();
    let horizontal_move = if offset.x > 0 {
        OFFSET_RIGHT
    } else {
        OFFSET_LEFT
    };
    let vertical_move = if offset.y > 0 { OFFSET_DOWN } else { OFFSET_UP };
    for _ in 0..offset.y.abs() {
        vertical.push(vertical_move);
    }
    for _ in 0..offset.x.abs() {
        horizontal.push(horizontal_move);
        vertical.push(horizontal_move);
    }
    for _ in 0..offset.y.abs() {
        horizontal.push(vertical_move);
    }
    if horizontal == vertical {
        vec![horizontal]
    } else {
        vec![horizontal, vertical]
    }
}

fn symbols_with_activate(path: &Vec<Offset>) -> Vec<u8> {
    let mut res = path.iter().cloned().map(offset_to_symbol).collect_vec();
    res.push(b'A');
    res
}

fn offset_to_symbol(offset: Offset) -> u8 {
    match offset {
        OFFSET_UP => b'^',
        OFFSET_DOWN => b'v',
        OFFSET_LEFT => b'<',
        OFFSET_RIGHT => b'>',
        _ => unreachable!(),
    }
}

fn complexity(code: &Code, code_len: u64) -> Result<u64> {
    Ok(code_len
        * String::from_utf8(
            code.iter()
                .filter(|b| b.is_ascii_digit())
                .cloned()
                .collect_vec(),
        )
        .unwrap()
        .parse::<u64>()?)
}

fn parse_input() -> Result<Vec<Code>> {
    Ok(stdin()
        .lines()
        .map(|line| -> Result<_> { Ok(line?.as_bytes().to_owned()) })
        .try_collect()?)
}
