use std::cmp::min;
use std::collections::HashMap;
use std::io;
use anyhow::{Result, Ok, Context, bail};

struct SpringLine {
    springs: Vec<Spring>,
    groups: Vec<usize>,
}

#[derive(Eq, Hash, PartialEq)]
struct CacheKey {
    from_spring_i: usize,
    from_group_i: usize,
}

impl SpringLine {
    fn arrangement_count(&self) -> u64 {
        let mut cache = HashMap::<CacheKey, u64>::new();
        self.arrangement_count_inner(0, 0, &mut cache)
    }

    fn arrangement_count_inner(&self, from_spring_i: usize, from_group_i: usize, cache: &mut HashMap<CacheKey, u64>) -> u64 {
        if from_group_i >= self.groups.len() {
            if from_spring_i >= self.springs.len() {
                return 1;
            }
            // no damaged groups left - it is a valid permutation if only unknown or operational springs remain
            return match self.springs[from_spring_i..].iter().all(|s| !matches!(s, Spring::Damaged)) {
                true => 1,
                false => 0
            };
        }

        if from_spring_i >= self.springs.len() {
            return 0;
        }

        let minimum_required = self.groups.len() - from_group_i - 1 + self.groups[from_group_i..].iter().sum::<usize>();
        if minimum_required > self.springs.len() - from_spring_i {
            return 0;
        }

        let group_size = self.groups[from_group_i];

        let limit_last_start = self.springs.len() - group_size;
        let last_possible_start = from_spring_i + self.springs[from_spring_i..]
            .iter()
            .position(|s| matches!(s, Spring::Damaged))
            .unwrap_or(limit_last_start);
        let last_possible_start = min(limit_last_start, last_possible_start);

        let mut sum: u64 = 0;
        for start in from_spring_i..=last_possible_start {
            // all springs in planned range have to be damaged or unknown
            if !self.springs[start..start + group_size]
                .iter()
                .all(|s| !matches!(s,  Spring::Operational)) {
                continue;
            }
            // and the very next must be either out of bounds, or not damaged
            match self.springs.get(start + group_size) {
                None => {}
                Some(s) => {
                    if matches!(s, Spring::Damaged) {
                        continue;
                    }
                }
            }

            let key = CacheKey{
                from_spring_i: start + group_size + 1,
                from_group_i: from_group_i + 1,
            };
            let count_opt = cache.get(&key);
            let count = match count_opt {
                None => {
                    let count = self.arrangement_count_inner(
                        start + group_size + 1,
                        from_group_i + 1,
                        cache
                    );
                    cache.insert(key, count);
                    count
                }
                Some(v) => *v
            };

            sum += count;
        }

        sum
    }


    fn unfold(&self, multiplier: usize) -> Self {
        let mut springs: Vec<Spring> = Vec::new();
        let mut first = true;
        for _ in 0..multiplier {
            if first {
                first = false;
            } else {
                springs.push(Spring::Unknown);
            }
            springs.extend(&self.springs);
        }
        Self {
            springs,
            groups: self.groups.repeat(multiplier),
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
        let spring_line = spring_line.unfold(5);
        let count = spring_line.arrangement_count();
        sum += count;
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
        groups: groups_str.split(",").map(|s| Ok(s.parse::<usize>()?)).collect::<Result<_>>()?,
    })
}

