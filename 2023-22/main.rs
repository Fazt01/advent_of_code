use std::cmp::{max, min};
use std::io;
use std::ops::{RangeInclusive};
use anyhow::{Result, Ok, Context};
use once_cell::sync::Lazy;
use regex::Regex;

#[derive(Clone, Debug, Eq, PartialEq)]
struct State {
    bricks: Vec<Brick>,
}

impl State {
    fn brick_xy_slice(&self, brick: &Brick, x_slice: &RangeInclusive<i64>, y_slice: &RangeInclusive<i64>) -> Option<Brick> {
        let new_x = *max(x_slice.start(), brick.x.start())..=*min(x_slice.end(), brick.x.end());
        let new_y = *max(y_slice.start(), brick.y.start())..=*min(y_slice.end(), brick.y.end());
        if new_x.is_empty() || new_y.is_empty() {
            return None;
        }
        Some(Brick {
            x: new_x,
            y: new_y,
            z: brick.z.clone(),
        })
    }

    fn fall(&mut self) {
        let old_bricks: Vec<Brick> = self.bricks.clone();
        let mut low_to_high = (0..old_bricks.len()).collect::<Vec<_>>();
        low_to_high.sort_by_key(|&i| old_bricks[i].z.start());
        for (j, (i, brick)) in low_to_high.iter().map(|&i| (i, &old_bricks[i])).enumerate() {
            let mut ground_for_brick: i64 = 0;
            for other_brick in low_to_high[0..j].iter().map(|&i| &self.bricks[i]) {
                if let Some(other_brick_slice) = self.brick_xy_slice(other_brick, &brick.x, &brick.y) {
                    let new_ground = *other_brick_slice.z.end();
                    if new_ground > ground_for_brick {
                        ground_for_brick = new_ground;
                    }
                }
            }
            let new_z_low = ground_for_brick + 1;
            self.bricks[i] = Brick {
                x: brick.x.clone(),
                y: brick.y.clone(),
                z: new_z_low..=new_z_low + (brick.z.end() - brick.z.start()),
            }
        }
    }

    fn can_disintegrate(&self) -> i64 {
        let mut sum = 0;
        for i in 0..self.bricks.len() {
            let supports = self.supports_total(i);
            if supports == 0 {
                sum += 1;
            }
        }
        sum
    }

    fn supports_total(&self, i: usize) -> i64 {
        let mut cloned = self.clone();
        cloned.bricks.remove(i);
        let before_fall = cloned.clone();
        cloned.fall();
        cloned.bricks.iter().zip(before_fall.bricks.iter())
            .filter(|(a,b)| a != b)
            .count() as i64
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Brick {
    x: RangeInclusive<i64>,
    y: RangeInclusive<i64>,
    z: RangeInclusive<i64>,
}

fn main() -> Result<()> {
    Lazy::force(&RE);

    let mut state = parse()?;

    state.fall();

    let sum = state.can_disintegrate();
    println!("{sum}");

    let mut sum = 0;
    for i in 0..state.bricks.len() {
        sum += state.supports_total(i)
    }
    println!("{sum}");

    Ok(())
}

static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\d+),(\d+),(\d+)~(\d+),(\d+),(\d+)").unwrap());

fn parse() -> Result<State> {
    let stdin = io::stdin();
    let mut bricks: Vec<Brick> = Vec::new();
    for line in stdin.lines() {
        let line = line?;
        let (_, groups) = RE.captures(&line).context("invalid line")?.extract::<6>();
        let nums = groups.iter().map(|x| Ok(x.parse::<i64>()?)).collect::<Result<Vec<_>>>()?;
        bricks.push(Brick {
            x: nums[0]..=nums[3],
            y: nums[1]..=nums[4],
            z: nums[2]..=nums[5],
        })
    }
    Ok(State {
        bricks,
    })
}
