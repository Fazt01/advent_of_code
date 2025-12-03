use anyhow::Result;
use std::io::stdin;

fn main() -> Result<()> {
    let banks = parse_input()?;

    let mut sum_part_1 = 0;
    let mut sum_part_2 = 0;

    for bank in banks {
        sum_part_1 += bank_joltage(&bank, 2);
        sum_part_2 += bank_joltage(&bank, 12);
    }

    println!("{}", sum_part_1);
    println!("{}", sum_part_2);

    Ok(())
}

fn bank_joltage(bank: &[u8], digits: usize) -> i64 {
    let mut first_usable_index = 0;
    let mut result: i64 = 0;
    for digit in 0..digits {
        let digit_candidates = &bank[first_usable_index..bank.len() - digits + digit + 1];
        let max_digit_position = first_usable_index
            + digit_candidates
                .iter()
                .position(|&v| v == *digit_candidates.iter().max().unwrap())
                .unwrap();
        first_usable_index = max_digit_position + 1;
        result *= 10;
        result += bank[max_digit_position] as i64;
    }
    result
}

fn parse_input() -> Result<Vec<Vec<u8>>> {
    stdin()
        .lines()
        .map(|line| -> Result<Vec<u8>> {
            let line = line?;
            Ok(line.as_bytes().iter().map(|&c| c - b'0').collect())
        })
        .collect()
}
