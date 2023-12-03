use std::{collections, io};
use std::cmp::Reverse;
use anyhow::Context;

fn main() -> Result<(), anyhow::Error> {
    let stdin = io::stdin();
    let mut sums: Vec<i32> = Vec::new();
    let mut sum = 0;
    for line in stdin.lines() {
        let line = line?;
        if line.is_empty() {
            sums.push(sum);
            sum = 0;
        } else {
            sum += line.parse::<i32>()?;
        }
    }
    if sum != 0 {
        sums.push(sum);
    }

    let mut heap = collections::BinaryHeap::new();
    for sum in sums {
        heap.push(Reverse(sum));
        if heap.len() > 3 {
            heap.pop();
        }
    }

    println!("{}", heap.pop().context("1")?.0+heap.pop().context("2")?.0+heap.pop().context("3")?.0);
    Ok(())
}