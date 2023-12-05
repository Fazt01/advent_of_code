use std::cmp::{min};
use std::io;
use std::ops::{Index, Range};
use anyhow::{bail, Context, Result, Ok};

struct Input {
    seeds: Vec<u64>,
    mappings: Vec<Mappings>,
}

struct Mappings {
    name: String,
    mapping: RangesMap,
}

struct RangesMap {
    ranges: Vec<RangeMap>,
}

impl RangesMap {
    // part 1
    fn map_num(&self, input: u64) -> u64 {
        let search_result = self.ranges.as_slice().binary_search_by_key(&input, |x| x.src);
        let candidate_range_map: Option<&RangeMap> = match search_result {
            Result::Ok(i) => { Some(self.ranges.index(i)) }
            // If exact match not found, the only candidate is exactly previous item, that could have
            // a .len spanning the searched num.
            Err(i) => {
                if i == 0 {
                    None
                } else {
                    Some(self.ranges.index(i - 1))
                }
            }
        };
        match candidate_range_map {
            None => { input }
            Some(range_map) => {
                if range_map.src <= input && ((input) < (range_map.src) + (range_map.len)) {
                    range_map.map_num(input)
                } else {
                    input
                }
            }
        }
    }

    // part 2
    fn map_range(&self, input: Range<u64>) -> Vec<Range<u64>> {
        let search_result = self.ranges.as_slice().binary_search_by_key(&input.start, |x| x.src);
        let relevant_maps_start_index: usize = match search_result {
            Result::Ok(i) => { i }
            Err(i) => {
                if i == 0 {
                    0
                } else {
                    if input.start < self.ranges[i - 1].src + self.ranges[i - 1].len {
                        i - 1
                    } else {
                        i
                    }
                }
            }
        };

        let mut result: Vec<Range<u64>> = Vec::new();
        let mut last_break_point = input.start;
        for relevant_range in self.ranges.iter().skip(relevant_maps_start_index) {
            if last_break_point >= input.end {
                return result;
            }
            if last_break_point < relevant_range.src {
                // unmapped sub-range
                result.push(last_break_point..min(input.end, relevant_range.src));
                last_break_point = min(input.end, relevant_range.src)
            }
            if last_break_point >= input.end {
                return result;
            }
            // mapped sub-range
            result.push(
                relevant_range.map_num(last_break_point)
                    ..relevant_range.map_num(min(input.end, relevant_range.src + relevant_range.len)));
            last_break_point = min(input.end, relevant_range.src + relevant_range.len)
        }
        if last_break_point < input.end {
            // last unmapped sub-range
            result.push(last_break_point..input.end);
        }
        result
    }
}

struct RangeMap {
    dest: u64,
    src: u64,
    len: u64,
}

impl RangeMap {
    fn map_num(&self, input: u64) -> u64 {
        input + self.dest - self.src // need to + before - to avoid negative overflow of unsigned type
    }
}

fn main() -> Result<()> {
    let input = parse()?;

    // part 1
    let mapped = map_through_all(&input);
    println!("{}", mapped.iter().min().context("no result")?);

    // part 2
    let mapped = map_ranges_through_all(&input);
    println!("{}", mapped.iter().map(|x| x.start).min().context("no result")?);

    Ok(())
}

fn map_through_all(input: &Input) -> Vec<u64> {
    return input.seeds.iter().map(|x| {
        let mut x = *x;
        for step in &input.mappings {
            x = step.mapping.map_num(x)
        }
        x
    }).collect();
}

fn map_ranges_through_all(input: &Input) -> Vec<Range<u64>> {
    let chunks = input.seeds.chunks_exact(2);
    let ranges = chunks.map(|x| x[0]..x[0] + x[1]);

    let mut current_ranges = ranges.collect::<Vec<Range<u64>>>();

    for mapping in &input.mappings {
        current_ranges = current_ranges.iter().
            flat_map(|x| {
                mapping.mapping.map_range(x.clone())
            }).
            collect();
    }

    current_ranges
}

fn parse() -> Result<Input> {
    let stdin = io::stdin();
    let mut lines = stdin.lines();
    let first = lines.next();
    let seeds = first.
        context("missing first line")??.
        strip_prefix("seeds:").context("missing first line header")?.
        split_whitespace().
        map(|s| Ok(s.parse::<u64>()?)).
        collect::<Result<Vec<_>>>().context("parsing first line")?;

    let mut mapping: Vec<Mappings> = Vec::new();
    for line in lines {
        let line = line.context("parsing mappings line")?;
        if line.is_empty() {
            continue;
        }

        if line.starts_with(char::is_alphabetic) {
            let mapping_header_str = line.split_once(':').context("missing ':' in mapping header line")?.0;
            let name = mapping_header_str.split('-').skip(2).next().context("missing mapping destination")?;

            mapping.push(Mappings {
                name: name.to_string(),
                mapping: RangesMap {
                    ranges: Vec::new(),
                },
            });
            continue;
        }

        let nums: Vec<u64> = line.
            split_whitespace().
            map(|s| Ok(s.parse::<u64>()?)).
            collect::<Result<_>>()?;
        if nums.len() != 3 {
            bail!("unexpected mapping length")
        }

        let start_dest = nums[0];
        let start_src = nums[1];
        let len = nums[2];
        mapping.
            last_mut().context("mapping nums before mapping header")?.
            mapping.ranges.push(
            RangeMap {
                dest: start_dest,
                src: start_src,
                len,
            });
    }

    for mapping in &mut mapping {
        mapping.mapping.ranges.sort_by_key(|x| x.src)
    }

    Ok(Input {
        seeds,
        mappings: mapping,
    })
}

#[cfg(test)]
mod tests {
    use once_cell::sync::Lazy;
    use crate::{RangeMap, RangesMap};

    static RANGES_MAP: Lazy<RangesMap> = Lazy::new(|| RangesMap {
        ranges: vec![
            RangeMap {
                dest: 100,
                src: 50,
                len: 10,
            }
        ]
    });

    #[test]
    fn before_unmapped() {
        assert_eq!(RANGES_MAP.map_range(45..50), vec![45..50])
    }

    #[test]
    fn after_unmapped() {
        assert_eq!(RANGES_MAP.map_range(60..65), vec![60..65])
    }

    #[test]
    fn start_range_mapped() {
        assert_eq!(RANGES_MAP.map_range(50..55), vec![100..105])
    }

    #[test]
    fn end_range_mapped() {
        assert_eq!(RANGES_MAP.map_range(55..60), vec![105..110])
    }

    #[test]
    fn part_through_start() {
        assert_eq!(RANGES_MAP.map_range(45..55), vec![45..50, 100..105])
    }

    #[test]
    fn part_through_end() {
        assert_eq!(RANGES_MAP.map_range(55..65), vec![105..110, 60..65])
    }
}