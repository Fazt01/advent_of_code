use std::cmp::Ordering;
use std::io::{stdin, Read};
use anyhow::{Result};
use once_cell::sync::Lazy;
use regex::Regex;

static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"mul\((\d{1,3}),(\d{1,3})\)").unwrap());
const DO_STR: &str = "do()";
const DONT_STR: &str = "don't()";

fn main() -> Result<()> {
    let mut buf = "".to_string();
    stdin().read_to_string(&mut buf)?;

    println!("{}", part2(&buf)?);

    Ok(())
}

fn part1(s: &str) -> Result<i32> {
    let mut sum = 0;

    for capture in RE.captures_iter(&s) {
        let mul = capture[1].parse::<i32>()? * capture[2].parse::<i32>()?;
        sum += mul;
    }

    Ok(sum)
}

fn part2(s: &str) -> Result<i32> {
    let mut sum = 0;
    let mut capture = RE.captures(&s);
    let mut capture_ref = capture.as_ref();
    let mut next_mul_i = capture_ref.map(|c| c.get(0).unwrap().start());
    let mut next_do_i = s.find(DO_STR);
    let mut next_dont_i = s.find(DONT_STR);
    let mut enabled = true;
    while let Some(next_mul_i_some) = next_mul_i {
        let (idx, _) = [next_mul_i, next_do_i, next_dont_i].into_iter().enumerate().min_by(
            |(idx_i, next_i), (idx_j, next_j)| cmp_idx_none_greatest((*idx_i, *next_i), (*idx_j, *next_j))
        ).unwrap();

        match idx {
            0 => {
                let capture_some = capture_ref.unwrap();
                if enabled {
                    let mul = capture_some[1].parse::<i32>()? * capture_some[2].parse::<i32>()?;
                    sum += mul;
                }
                let scan_i = next_mul_i_some + capture_some.get(0).unwrap().len();
                capture = RE.captures(&s[scan_i..]);
                capture_ref = capture.as_ref();
                next_mul_i = capture_ref.map(|c| scan_i + c.get(0).unwrap().start());
            }
            1 => {
                enabled = true;
                let scan_i = next_do_i.unwrap() + DO_STR.len();
                next_do_i = s[scan_i..].find(DO_STR).map(|i| scan_i + i);
            }
            2 => {
                enabled = false;
                let scan_i = next_dont_i.unwrap() + DONT_STR.len();
                next_dont_i = s[scan_i..].find(DONT_STR).map(|i| scan_i + i);
            }
            _ => unreachable!()
        }
    }

    Ok(sum)
}

fn cmp_idx_none_greatest((_, i): (usize, Option<usize>), (_, j): (usize, Option<usize>)) -> Ordering{
    match i {
        None => {
            match j {
                None => Ordering::Equal,
                Some(_) => Ordering::Greater
            }
        }
        Some(i) => {
            match j {
                None => Ordering::Less,
                Some(j) => i.cmp(&j)
            }
        }
    }
}