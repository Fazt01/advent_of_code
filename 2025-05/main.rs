use anyhow::{Context, Result};
use std::cmp::max;
use std::io::stdin;
use std::ops::RangeInclusive;

#[derive(Default)]
struct Database {
    fresh_ranges: Vec<RangeInclusive<i64>>,
    available: Vec<i64>,
}

#[derive(Default)]
struct MergedRange {
    ranges: Vec<RangeInclusive<i64>>,
}

impl MergedRange {
    fn from_iter<I: IntoIterator<Item = RangeInclusive<i64>>>(iter: I) -> Self {
        let mut result = Self::default();
        for range in iter {
            result.add(range)
        }
        result
    }

    fn add(&mut self, range: RangeInclusive<i64>) {
        let new_start_i = would_sort_index(&self.ranges, &range, |range| *range.start());
        let new_end_i = would_sort_index(&self.ranges, &range, |range| *range.end());
        let start_overlaps = if new_start_i == 0 {
            false
        } else {
            *(&self.ranges[new_start_i - 1]).end() + 1 >= *range.start()
        };
        let end_overlaps = if new_end_i == self.ranges.len() {
            false
        } else {
            *range.end() + 1 >= *self.ranges[new_end_i].start()
        };
        if new_end_i > new_start_i {
            self.ranges.drain(new_start_i..new_end_i);
        }
        self.ranges.insert(new_start_i, range.clone());
        if end_overlaps && new_end_i >= new_start_i {
            let merged_end = *range.start()..=*self.ranges[new_start_i + 1].end();
            self.ranges[new_start_i] = merged_end;
            self.ranges.remove(new_start_i + 1);
        }
        if start_overlaps {
            let merged_start = *self.ranges[new_start_i - 1].start()
                ..=max(
                    *self.ranges[new_start_i - 1].end(),
                    *self.ranges[new_start_i].end(),
                );
            self.ranges[new_start_i] = merged_start;
            self.ranges.remove(new_start_i - 1);
        }
    }

    fn size(&self) -> i64 {
        let mut result = 0;
        for range in &self.ranges {
            result += range.end() - range.start() + 1;
        }
        result
    }
}

fn would_sort_index<T, B: Ord, F: FnMut(&T) -> B>(
    ranges: &[T],
    new_item: &T,
    mut key_fn: F,
) -> usize {
    match ranges.binary_search_by_key(&key_fn(new_item), key_fn) {
        Ok(x) => x + 1,
        Err(x) => x,
    }
}

fn main() -> Result<()> {
    let database = parse_input()?;

    let part1_count = database
        .available
        .iter()
        .filter(|&id| database.fresh_ranges.iter().any(|range| range.contains(id)))
        .count();

    println!("{}", part1_count);

    let part2_count = MergedRange::from_iter(database.fresh_ranges.iter().cloned()).size();

    println!("{}", part2_count);

    Ok(())
}

fn parse_input() -> Result<Database> {
    let mut available_phase = false;
    let mut result = Database::default();
    for line in stdin().lines() {
        let line = line?;
        if !available_phase {
            if line == "" {
                available_phase = true;
                continue;
            }
            let mut range_strs = line.splitn(2, "-");
            let range_start = range_strs
                .next()
                .context("expected range start")?
                .parse()
                .with_context(|| format!("parsing range start, got '{}'", line))?;
            let range_end = range_strs
                .next()
                .context("expected range end")?
                .parse()
                .with_context(|| format!("parsing range end, got '{}'", line))?;
            result.fresh_ranges.push(range_start..=range_end)
        } else {
            result.available.push(
                line.parse()
                    .with_context(|| format!("parsing available ingredient, got '{}'", line))?,
            );
        }
    }
    Ok(result)
}
