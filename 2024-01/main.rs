use std::collections::BTreeMap;
use std::io::stdin;
use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use regex::Regex;

static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\d+)\s+(\d+)").unwrap());

type Lists = [BTreeMap<i32, i32>; 2];
fn main() -> Result<()>{
    Lazy::force(&RE);

    let mut lists = parse_input()?;

    // part 2
    let mut sum = 0;
    let (left, right) = lists.split_at_mut(1);
    for (location_id, left_count) in left[0].iter() {
        let right_count = right[0].get(location_id).unwrap_or(&0);
        let similarity = location_id * left_count * right_count;
        sum += similarity;
        //println!("location {} found {} and {} times in right found for total similarity of {}", location_id, left_count, right_count, similarity);
    }

    println!("{}", sum);

    // part 1
    let mut sum = 0;

    while !lists[0].is_empty() {
        let (left, right) = lists.split_at_mut(1);
        let mut least1 = left[0].first_entry().unwrap();
        let mut least2 = right[0].first_entry().context("second list empty while the first list is not empty")?;

        let distance = least1.key().abs_diff(*least2.key());

        //println!("{} and {} found for total distance of {}", least1.key(), least2.key(), distance);

        sum += distance;

        let val1 = least1.get_mut();
        *val1 -= 1;
        if *val1 <= 0 {
            least1.remove();
        }

        let val2 = least2.get_mut();
        *val2 -= 1;
        if *val2 <= 0 {
            least2.remove();
        }
    }

    println!("{}", sum);

    Ok(())
}

fn parse_input() -> Result<Lists> {
    let mut res: Lists = Default::default();

    for (i, line) in stdin().lines().enumerate() {
        let line = line?;
        let captures = RE.captures(&line).with_context(|| format!("No match on line {}:\n{}", i+1, line))?;
        for (list, parsed_str) in res.iter_mut().zip([&captures[1], &captures[2]].into_iter()) {
            let parsed_int = parsed_str.parse::<i32>().with_context(|| format!("Could not parse into i32: {}", parsed_str))?;
            list.entry(parsed_int).and_modify(|x| *x += 1).or_insert(1);
        }
    }

    Ok(res)
}