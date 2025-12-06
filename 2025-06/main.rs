use anyhow::{bail, Context, Result};
use itertools::Itertools;
use std::io::stdin;

#[derive(Default, Debug)]
struct Worksheet {
    cols: Vec<WorksheetColumn>,
}

impl Worksheet {
    fn sum(&self) -> i64 {
        let mut result = 0;
        for column in &self.cols {
            let mut sum = match column.operation {
                Operation::Add => 0,
                Operation::Mul => 1,
            };
            for value in &column.nums {
                match column.operation {
                    Operation::Add => sum += value,
                    Operation::Mul => sum *= value,
                }
            }
            result += sum
        }
        result
    }
}

#[derive(Default, Debug)]
struct WorksheetColumn {
    nums: Vec<i64>,
    operation: Operation,
}

#[derive(Default, Debug)]
enum Operation {
    #[default]
    Add,
    Mul,
}

fn main() -> Result<()> {
    // only one of these can be run, as they consume stdin input
    // let worksheet = parse_input()?;
    let worksheet2 = parse_input_part2()?;

    // println!("{}", worksheet.sum());
    println!("{}", worksheet2.sum());

    Ok(())
}

fn parse_input() -> Result<Worksheet> {
    let mut result = Worksheet::default();
    for line in stdin().lines() {
        let line = line?;
        let split = line.split_whitespace().collect_vec();
        if result.cols.len() == 0 {
            for _ in &split {
                result.cols.push(WorksheetColumn::default())
            }
        }
        for (i, &value_str) in split.iter().enumerate() {
            if value_str
                .chars()
                .next()
                .context("empty value")?
                .is_numeric()
            {
                result.cols[i].nums.push(
                    value_str
                        .parse()
                        .with_context(|| format!("parsing a numeric value, got '{}'", value_str))?,
                );
            } else {
                result.cols[i].operation = match value_str {
                    "+" => Operation::Add,
                    "*" => Operation::Mul,
                    s => bail!("parsing an operation, got '{}'", s),
                }
            }
        }
    }
    Ok(result)
}

fn parse_input_part2() -> Result<Worksheet> {
    let mut result = Worksheet::default();
    result.cols.push(WorksheetColumn::default());
    let lines = stdin().lines().collect::<Result<Vec<String>, _>>()?;
    let mut col_i = 0;
    for byte_i in 0..lines[0].len() {
        let mut num: i64 = 0;
        let mut is_col_separation = true;
        for line in &lines[0..lines.len() - 1] {
            let byte = line.as_bytes()[byte_i];
            if !byte.is_ascii_whitespace() {
                if !byte.is_ascii_digit() {
                    bail!("expected a digit, got byte {}", byte)
                }
                is_col_separation = false;
                num *= 10;
                num += (byte - b'0') as i64
            }
        }

        match lines[lines.len() - 1]
            .as_bytes()
            .get(byte_i)
            .unwrap_or(&b' ')
        {
            b'+' => result.cols[col_i].operation = Operation::Add,
            b'*' => result.cols[col_i].operation = Operation::Mul,
            b' ' => {}
            b => bail!("parsing an operation, got byte {}", b),
        }
        if is_col_separation {
            col_i += 1;
            result.cols.push(WorksheetColumn::default())
        } else {
            result.cols[col_i].nums.push(num);
        }
    }

    Ok(result)
}
