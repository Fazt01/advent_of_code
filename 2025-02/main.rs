use anyhow::{Context, Result};
use std::io::{stdin, Read};
use std::ops::RangeInclusive;

fn main() -> Result<()> {
    let input = parse_input()?;

    let mut sum_part_1 = 0;
    let mut sum_part_2 = 0;

    for range in input {
        for id in range {
            if is_invalid_id_part1(id) {
                sum_part_1 += id
            }
            if is_invalid_id_part2(id) {
                sum_part_2 += id
            }
        }
    }

    println!("{}", sum_part_1);
    println!("{}", sum_part_2);

    Ok(())
}

fn is_invalid_id_part1(v: i64) -> bool {
    let s = v.to_string();
    if s.len() % 2 != 0 {
        return false;
    }
    let half_len = s.len() / 2;
    let magnitude = 10_i64.pow(half_len as u32);
    v / magnitude == v % magnitude
}

fn is_invalid_id_part2(v: i64) -> bool {
    let s = v.to_string();
    let half_len = s.len() / 2;
    'outer: for sequence_len in 1..=half_len {
        if s.len() % sequence_len != 0 {
            continue;
        }
        let magnitude = 10_i64.pow(sequence_len as u32);
        let repeated = v % magnitude;
        let mut remaining = v;
        while remaining > 0 {
            if remaining % magnitude != repeated {
                continue 'outer;
            }
            remaining /= magnitude
        }
        return true;
    }
    false
}

fn parse_input() -> Result<Vec<RangeInclusive<i64>>> {
    let mut buf: String = Default::default();
    stdin().read_to_string(&mut buf)?;

    buf.split(",")
        .map(|s| -> Result<RangeInclusive<i64>> {
            let mut range_strs = s.splitn(2, "-");
            let range_start = range_strs.next().context("expected range start")?.parse()?;
            let range_end = range_strs.next().context("expected range end")?.parse()?;
            Ok(range_start..=range_end)
        })
        .collect()
}
