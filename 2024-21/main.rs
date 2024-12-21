use anyhow::Result;
use grid::{Coord, Grid, Offset, OFFSET_DOWN, OFFSET_LEFT, OFFSET_RIGHT, OFFSET_UP};
use itertools::{repeat_n, Itertools};
use std::collections::HashMap;
use std::io::stdin;
use std::rc::Rc;

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

const KEYBOARD_COUNT: usize = 3;

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
    let keyboards = vec![&final_keypad]
        .into_iter()
        .chain(repeat_n(&intermediate_keypad, KEYBOARD_COUNT))
        .collect_vec();

    let mut sum = 0;
    for code in &input {
        let state = State {
            keyboards: vec![Some(KeyboardState {
                to_input: Rc::from(code.clone()),
                starting: final_keypad.coords[&b'A'],
            })]
            .into_iter()
            .chain(repeat_n(None, KEYBOARD_COUNT))
            .collect_vec(),
            output_start: 0,
        };

        let mut output: Vec<u8> = vec![];
        let mut states = vec![state];
        let mut shortest: Option<Vec<u8>> = None;
        while let Some(state) = states.pop() {
            let new_states = expand_state(&state, &mut output, &keyboards);
            // println!("{state:?}");
            if new_states.is_empty() {
                shortest = Some(match shortest {
                    None => output.clone(),
                    Some(previous_shortest) => {
                        if previous_shortest.len() > output.len() {
                            output.clone()
                        } else {
                            previous_shortest
                        }
                    }
                });
            }
            states.extend(new_states);
        }
        let complexity = complexity(code, shortest.as_ref().unwrap())?;
        sum += complexity;
        println!("{complexity} {}", shortest.as_ref().unwrap().len());
        println!(
            "{}: {}",
            String::from_utf8(code.clone())?,
            String::from_utf8(shortest.unwrap())?
        );
    }
    println!("{sum}");

    Ok(())
}

#[derive(Debug, Clone)]
struct KeyboardState {
    to_input: Rc<[u8]>,
    starting: Coord,
}

#[derive(Debug, Clone)]
struct State {
    keyboards: Vec<Option<KeyboardState>>,
    output_start: usize,
}

fn expand_state(state: &State, output: &mut Vec<u8>, keyboards: &[&Keyboard]) -> Vec<State> {
    for (i, keyboard_state) in state.keyboards.iter().enumerate() {
        if keyboard_state.is_none() {
            if i == 0 {
                return vec![];
            }
            let mut res = vec![];
            let prev_keyboard_dest = state.keyboards[i - 1].as_ref().unwrap().to_input.first();
            if let Some(prev_keyboard_dest) = prev_keyboard_dest {
                let dest_coord = keyboards[i - 1].coords[prev_keyboard_dest];
                let src_coord = state.keyboards[i - 1].as_ref().unwrap().starting;
                let offset = dest_coord - src_coord;
                let paths = get_paths(offset);
                'paths: for path in &paths {
                    let mut gap_test_coord = src_coord;
                    for &offset in path {
                        gap_test_coord = gap_test_coord + offset;
                        if keyboards[i - 1].grid[gap_test_coord] == b'.' {
                            continue 'paths;
                        }
                    }
                    let next_input = symbols_with_activate(path);
                    let mut next_state = state.clone();
                    next_state.keyboards[i] = Some(KeyboardState {
                        to_input: Rc::from(next_input),
                        starting: keyboards[i].coords[&b'A'],
                    });
                    let prev_keyboard = next_state.keyboards[i - 1].as_mut().unwrap();
                    prev_keyboard.to_input = prev_keyboard.to_input[1..].into();
                    prev_keyboard.starting = dest_coord;
                    res.push(next_state);
                }
                return res;
            }
            let mut next_state = state.clone();
            if i - 1 == 0 {
                return vec![];
            }
            next_state.keyboards[i - 1] = None;
            return vec![next_state];
        }
    }

    let manual_input = state
        .keyboards
        .last()
        .unwrap()
        .as_ref()
        .unwrap()
        .to_input
        .as_ref();
    output.truncate(state.output_start);
    output.extend(manual_input.iter());
    let mut next_state = state.clone();
    *next_state.keyboards.last_mut().unwrap() = None;
    next_state.output_start += manual_input.len();

    vec![next_state]
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

fn complexity(code: &Code, shortest: &Code) -> Result<u64> {
    Ok(shortest.len() as u64
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
