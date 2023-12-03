use std::collections::HashMap;
use std::io;
use anyhow::Error;

#[derive(Default)]
struct State {
    state: Vec<Vec<Cell>>,
    rows: i64,
    columns: i64,
}

enum Cell {
    Empty,
    Digit(DigitState),
    Symbol(u8),
}

struct DigitState {
    digit: u8,
    active: bool,
    // A digit state coords that last activated this DigitState - unique for each number,
    // as this cannot spread across number bounds.
    // Is  uniques for purposes of identifying unique parts/numbers next to gears
    last_seed: Option<CellCoordinate>,
    contributes_to_number: Option<u64>,
}

#[derive(Eq, Hash, PartialEq, Copy, Clone)]
struct CellCoordinate {
    row: i64,
    column: i64,
}

fn main() -> Result<(), Error> {
    let stdin = io::stdin();
    let mut state: State = Default::default();
    for line in stdin.lines() {
        let line = line?;
        let line_bytes = line.as_bytes();

        parse_line(line_bytes, &mut state);
    }

    activate_surrounding_symbol(&mut state);
    spread_digits_activation(&mut state);
    let sum = sum_active_digit_numbers(&mut state);
    let gear_sum = sum_2_gears(&state);

    // part 1
    println!("{}", sum);

    // part 2
    println!("{}", gear_sum);

    Ok(())
}

fn sum_2_gears(state: &State) -> i64 {
    let mut sum = 0;
    for row in 0..state.rows {
        for column in 0..state.columns {
            match state.state[row as usize][column as usize] {
                Cell::Symbol(b'*') => {
                    let mut parts = HashMap::<CellCoordinate, i64>::new();
                    for offset in SURROUNDING_OFFSET {
                        let offset_row = row + (offset.0 as i64);
                        let offset_column = column + (offset.1 as i64);
                        if offset_row < 0 || offset_row >= state.rows || offset_column < 0 || offset_column >= state.columns {
                            continue;
                        }
                        let cell: &Cell = &state.state[offset_row as usize][offset_column as usize];
                        match cell {
                            Cell::Digit(digit_state) if digit_state.contributes_to_number.is_some() => {
                                parts.insert(digit_state.last_seed.unwrap(), digit_state.contributes_to_number.unwrap() as i64);
                            }
                            _ => {}
                        }
                    }
                    if parts.len() == 2 {
                        sum += parts.values().product::<i64>()
                    }
                }
                _ => {}
            }
        }
    }
    sum
}

fn sum_active_digit_numbers(state: &mut State) -> u64 {
    let mut sum: u64 = 0;

    let mut number: u64 = 0;
    for row in 0..state.rows {
        number = 0;
        for column in 0..state.columns {
            match &state.state[row as usize][column as usize] {
                Cell::Digit(digit_state) if digit_state.active => {
                    number = number * 10 + (digit_state.digit as u64);
                }
                _ => {
                    // for part 2
                    for back_spread_column in (0..column).rev() {
                        match &mut state.state[row as usize][back_spread_column as usize] {
                            Cell::Digit(digit_state) => {
                                digit_state.contributes_to_number = Some(number);
                            }
                            _ => {
                                break;
                            }
                        }
                    }
                    sum += number;
                    number = 0;
                }
            }
        }
        // for part 2
        for back_spread_column in (0..state.columns).rev() {
            match &mut state.state[row as usize][back_spread_column as usize] {
                Cell::Digit(digit_state) => {
                    digit_state.contributes_to_number = Some(number);
                }
                _ => {
                    break;
                }
            }
        }
        sum += number;
        number = 0;
    }
    sum
}

fn spread_digits_activation(state: &mut State) {
    for row in 0..state.rows {
        for column in 0..state.columns {
            match &state.state[row as usize][column as usize] {
                Cell::Digit(digit_state) => {
                    if digit_state.active {
                        for back_spread_column in (0..column).rev() {
                            match &mut state.state[row as usize][back_spread_column as usize] {
                                Cell::Digit(digit_state) => {
                                    digit_state.active = true;
                                    digit_state.last_seed = Some(CellCoordinate {
                                        row,
                                        column,
                                    });
                                }
                                _ => {
                                    break;
                                }
                            }
                        }
                        for forward_spread_column in (column + 1..state.columns) {
                            match &mut state.state[row as usize][forward_spread_column as usize] {
                                Cell::Digit(digit_state) => {
                                    digit_state.active = true
                                }
                                _ => {
                                    break;
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

fn activate_surrounding_symbol(state: &mut State) {
    for row in 0..state.rows {
        for column in 0..state.columns {
            match state.state[row as usize][column as usize] {
                Cell::Symbol(_) => {
                    for offset in SURROUNDING_OFFSET {
                        let offset_row = row + (offset.0 as i64);
                        let offset_column = column + (offset.1 as i64);
                        if offset_row < 0 || offset_row >= state.rows || offset_column < 0 || offset_column >= state.columns {
                            continue;
                        }
                        let cell: &mut Cell = state.state.get_mut(offset_row as usize).unwrap().get_mut(offset_column as usize).unwrap();
                        match cell {
                            Cell::Digit(digit_state) => {
                                digit_state.active = true;
                                digit_state.last_seed = Some(CellCoordinate {
                                    row: offset_row,
                                    column: offset_column,
                                })
                            }
                            _ => {}
                        };
                    }
                }
                _ => {}
            }
        }
    }
}

static SURROUNDING_OFFSET: [(i32, i32); 8] = [
    (-1, 0),
    (-1, -1),
    (0, -1),
    (1, -1),
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
];

fn parse_line(line: &[u8], state: &mut State) {
    let mut row = Vec::<Cell>::with_capacity(state.columns as usize);
    for byte in line {
        let cell = match byte {
            b if *byte >= b'0' && *byte <= b'9' => {
                Cell::Digit(DigitState {
                    digit: b - b'0',
                    active: false,
                    last_seed: None,
                    contributes_to_number: None,
                })
            }
            _ if *byte == b'.' => {
                Cell::Empty
            }
            b => Cell::Symbol(*b)
        };
        row.push(cell)
    }
    state.columns += 1;
    if state.rows == 0 {
        state.rows = row.len() as i64;
    }
    state.state.push(row);
}