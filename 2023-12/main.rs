use std::io;
use anyhow::{Result, Ok, Context, bail};

struct SpringLine {
    springs: Vec<Spring>,
    groups: Vec<u64>,
}

impl SpringLine {
    fn arrangement_count(&self) -> u64 {
        let mut checked_springs = self.springs.clone();
        self.arrangement_count_rec(checked_springs.as_mut_slice(), 0)
    }

    fn arrangement_count_rec(&self, checked: &mut [Spring], from_i: usize) -> u64 {
        let mut sum = 0;
        for spring_i in from_i..checked.len() {
            if matches!(self.springs[spring_i], Spring::Unknown) {
                checked[spring_i] = Spring::Operational;
                sum += self.arrangement_count_rec(checked, spring_i + 1);
                checked[spring_i] = Spring::Damaged;
                sum += self.arrangement_count_rec(checked, spring_i + 1);
                return sum;
            }
        }
        return match self.matches_groups(checked) {
            true => 1,
            false => 0
        };
    }

    fn matches_groups(&self, checked: &[Spring]) -> bool {
        let mut group_i = 0;
        let mut damaged_row = 0;
        for spring in checked.iter().chain([Spring::Operational].iter()) {
            match spring {
                Spring::Unknown => { unreachable!("should already be filled") }
                Spring::Damaged => {
                    damaged_row += 1;
                }
                Spring::Operational => {
                    if damaged_row > 0 {
                        if group_i >= self.groups.len() {
                            return false;
                        }
                        if self.groups[group_i] != damaged_row {
                            return false;
                        }
                        group_i += 1;
                        damaged_row = 0;
                    }
                }
            }
        }

        group_i == self.groups.len()
    }

    fn unfold(&self, multiplier: u64) -> Self {
        Self{
            springs: self.springs.repeat(multiplier as usize),
            groups: self.groups.repeat(multiplier as usize),
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum Spring {
    Unknown,
    Damaged,
    Operational,
}

fn main() -> Result<()> {
    let stdin = io::stdin();
    let mut sum = 0;
    for line in stdin.lines() {
        let line = line?;
        let spring_line = parse_line(&line)?;
        // let spring_line = spring_line.unfold(5);
        sum += spring_line.arrangement_count();
    }

    println!("{}", sum);

    Ok(())
}

fn parse_line(line: &str) -> Result<SpringLine> {
    let (springs_str, groups_str) = line.split_once(" ").context("no split")?;
    Ok(SpringLine {
        springs: springs_str.chars().map(|c| Ok(match c {
            '#' => Spring::Damaged,
            '.' => Spring::Operational,
            '?' => Spring::Unknown,
            _ => bail!("invalid spring state")
        })).collect::<Result<_>>()?,
        groups: groups_str.split(",").map(|s| Ok(s.parse::<u64>()?)).collect::<Result<_>>()?,
    })
}

