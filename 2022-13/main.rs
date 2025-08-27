use anyhow::{bail, Context, Ok, Result};
use lib::str::StrExt;
use std::cmp::{Ordering, PartialOrd};
use std::io::stdin;
use std::str::FromStr;

#[derive(Debug)]
enum Value {
    Int(u64),
    List(Vec<Box<Value>>),
}

impl Default for Value {
    fn default() -> Self {
        Self::Int(0)
    }
}

impl Value {
    fn parse(s: &str) -> Result<Value> {
        let (value, len) = Self::parse_inner(s).with_context(|| format!("string '{s}'"))?;
        if len != s.len() {
            bail!("unexpected character {}", s.char_at(len))
        }
        Ok(value)
    }

    fn parse_inner(s: &str) -> Result<(Value, usize)> {
        let mut index = 0;
        if s.is_empty() {
            bail!("parsing empty string")
        }
        match s.char_at(0) {
            '[' => {
                let mut list: Vec<Box<Value>> = vec![];
                let mut start_i = 1;
                loop {
                    if s.char_at(start_i) == ']' {
                        return Ok((Value::List(list), start_i + 1));
                    }
                    let (value, len) = Self::parse_inner(&s[start_i..])
                        .with_context(|| format!("index {index}"))?;
                    if start_i + len >= s.len() {
                        bail!("list not terminated");
                    }
                    list.push(Box::new(value));
                    match s.char_at(start_i + len) {
                        ']' => return Ok((Value::List(list), start_i + len + 1)),
                        ',' => {
                            start_i = start_i + len + 1;
                            index += 1;
                            continue;
                        }
                        x => bail!("unexpected character '{}'", x),
                    }
                }
            }
            _ => {
                let int_s = match s.find(|c: char| !c.is_numeric()) {
                    None => s,
                    Some(end_i) => &s[..end_i],
                };
                Ok((
                    Value::Int(
                        int_s
                            .parse::<u64>()
                            .with_context(|| format!("index {index}"))?,
                    ),
                    int_s.len(),
                ))
            }
        }
    }
}

impl FromStr for Value {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        Value::parse(s)
    }
}

impl Eq for Value {}

impl Ord for Value {
    fn cmp(&self, other: &Self) -> Ordering {
        match (&self, other) {
            (Value::Int(lhs), Value::Int(rhs)) => lhs.cmp(rhs),
            (Value::List(lhs), Value::List(rhs)) => lhs.cmp(rhs),
            (Value::Int(lhs), Value::List(_)) => {
                Value::List(vec![Box::new(Value::Int(*lhs))]).cmp(other)
            }
            (Value::List(_), Value::Int(rhs)) => {
                self.cmp(&Value::List(vec![Box::new(Value::Int(*rhs))]))
            }
        }
    }
}

impl PartialEq<Self> for Value {
    fn eq(&self, other: &Self) -> bool {
        matches!(self.cmp(other), Ordering::Equal)
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn main() -> Result<()> {
    let pairs = parse()?;

    println!("{}", part1(&pairs));
    println!("{}", part2(&pairs));

    Ok(())
}

fn part1(pairs: &Vec<[Value; 2]>) -> u64 {
    pairs
        .iter()
        .enumerate()
        .filter(|(_, pair)| pair[0] < pair[1])
        .map(|(i, _)| i + 1)
        .sum::<usize>() as u64
}

#[derive(Debug)]
enum Marker {
    Original,
    Divider,
}

fn part2(pairs: &Vec<[Value; 2]>) -> u64 {
    let all_pairs: Vec<_> = pairs.iter().flatten().collect();

    let mut marked_pairs: Vec<_> = all_pairs.iter().map(|x| (Marker::Original, x)).collect();
    let div1 = &"[[2]]".parse::<Value>().unwrap();
    let div2 = &"[[6]]".parse::<Value>().unwrap();
    marked_pairs.push((Marker::Divider, &div1));
    marked_pairs.push((Marker::Divider, &div2));

    marked_pairs.sort_by_key(|(_, &x)| x);
    marked_pairs
        .into_iter()
        .enumerate()
        .filter(|(_, (m, _))| matches!(m, Marker::Divider))
        .map(|(i, _)| i + 1)
        .product::<usize>() as u64
}

fn parse() -> Result<Vec<[Value; 2]>> {
    let mut result: Vec<[Value; 2]> = vec![];
    let mut pair: [Value; 2] = Default::default();

    let mut i = 0;
    for line in stdin().lines() {
        let line = line?;
        if i >= 2 {
            if line == "" {
                result.push(pair);
                pair = Default::default();
                i = 0;
                continue;
            } else {
                bail!("expected blank line, got '{}'", line)
            }
        }
        pair[i] = line.parse()?;
        i += 1;
    }
    if i == 2 {
        result.push(pair);
    }

    Ok(result)
}
