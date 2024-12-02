use std::io::stdin;
use anyhow::{Result};
fn main() -> Result<()>{
    let input = parse_input()?;

    let result = input.into_iter().filter(|x| is_safe_part2(x)).count();

    println!("{result}");

    Ok(())
}

fn is_safe(level: &[i32]) -> bool {
    let increasing = level[1] > level[0];

    is_safe_direction(level, increasing)
}

fn is_safe_part2(level: &[i32]) -> bool {
    is_safe(level)
    || (0..level.iter().len()).any(|i| is_safe(&skip_nth(level, i)))
}

fn is_safe_direction(level: &[i32], increasing: bool) -> bool {
    level.windows(2).all(|window|
        is_safe_direction_pair(window, increasing)
    )
}

fn is_safe_direction_pair(pair: &[i32], increasing: bool) -> bool {
    (if increasing {
        pair[1] > pair[0]
    } else {
        pair[1] < pair[0]
    }) && (pair[0].abs_diff(pair[1]) <= 3)
}

fn skip_nth(vec: &[i32], n: usize) -> Vec<i32> {
    vec.iter().enumerate().filter(|&(i, _)| i != n).map(|(_, v)| *v).collect()
}

fn parse_input() -> Result<Vec<Vec<i32>>> {
    stdin().lines().map(|line| -> Result<Vec<i32>> {
        line?.split_whitespace().map(|x| Ok(x.parse::<i32>()?)).collect()
    }).collect()
}