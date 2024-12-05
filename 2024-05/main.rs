use std::collections::HashSet;
use anyhow::{bail, Context, Result};
use std::io::stdin;

struct Puzzle {
    rules: Vec<Rule>,
    unordered_lists: Vec<Vec<i32>>,
}

#[derive(Hash, Eq, PartialEq)]
struct Rule {
    lesser: i32,
    greater: i32,
}
fn main() -> Result<()> {
    let puzzle = parse_input()?;

    //part1
    println!("{}", order_and_sum_middle(&puzzle, true)?);
    //part2
    println!("{}", order_and_sum_middle(&puzzle, false)?);

    Ok(())
}

fn order_and_sum_middle(puzzle: &Puzzle, count_ordered: bool) -> Result<i32> {
    let mut sum = 0;

    for unordered_list in &puzzle.unordered_lists {
        let mut nums: HashSet<i32> = HashSet::from_iter(unordered_list.iter().cloned());
        let mut list_rules = HashSet::<_>::from_iter(
            puzzle.rules.iter()
                .filter(|&rule| nums.contains(&rule.lesser) && nums.contains(&rule.greater))
        );
        let mut ordered_list: Vec<i32> = Default::default();
        while !nums.is_empty() {
            // Find next least element. Such element is not greater than any other (of the remaining).
            let least_elem = nums.iter().filter(
                |&&candidate| !list_rules.iter().any(
                    |&rule| rule.greater == candidate
                )
            ).cloned()
                .next()
                .context("no least element found - remaining rules do not specify a valid partial ordering")?;
            nums.remove(&least_elem);
            list_rules.retain(|&rule| rule.lesser != least_elem);
            ordered_list.push(least_elem);
        }
        if ordered_list.eq(unordered_list) == count_ordered {
            sum += ordered_list[ordered_list.len()/2]
        }
    }

    Ok(sum)
}

fn parse_input() -> Result<Puzzle> {
    let mut result = Puzzle{
        rules: vec![],
        unordered_lists: vec![],
    };

    let mut rules = true;
    for line in stdin().lines() {
        let line = line?;
        if rules {
            if line.is_empty() {
                rules = false;
                continue
            }
            let rule_nums: Vec<i32> = line.split('|')
                .map(|s| Ok(s.parse()?))
                .collect::<Result<_>>()?;
            if rule_nums.len() != 2 {
                bail!("unexpected {} pipe separated rules in: {line}", rule_nums.len())
            }
            result.rules.push(Rule{
                lesser: rule_nums[0],
                greater: rule_nums[1]
            });
        } else {
            let nums: Vec<i32> = line.split(',')
                .map(|s| Ok(s.parse()?))
                .collect::<Result<_>>()?;
            result.unordered_lists.push(nums);
        }
    }

    Ok(result)
}