use anyhow::{Context, Result};
use crossterm::cursor::MoveTo;
use crossterm::{terminal::{EnterAlternateScreen}, ExecutableCommand};
use lib::grid::{Coord, Offset};
use once_cell::sync::Lazy;
use regex::Regex;
use std::io::{stdin, stdout, Write};
use std::thread::sleep;
use std::time::Duration;

static RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"p=(-?\d+),(-?\d+) v=(-?\d+),(-?\d+)").unwrap());

struct Robot {
    position: Coord,
    velocity: Offset,
}

const ROWS: i64 = 103;
const COLUMNS: i64 = 101;
fn main() -> Result<()> {
    let mut input = parse_input()?;

    let mut stdout = stdout();

    stdout.execute(EnterAlternateScreen)?;

    let mut i = 0;
    loop {
        i += 1;
        for robot in &mut input {
            // run_for(robot, 100)
            run_for(robot, 1)
        }
        redraw(&mut stdout, &input, i)?;
        sleep(Duration::from_millis(5000));
    }

    // println!("{}", count_quadrants(&input));
}

fn redraw<W: Write>(w: &mut W, robots: &Vec<Robot>, i: i32) -> Result<()> {
    w.execute(MoveTo(0,0))?;
    w.write_fmt(format_args!("{}\n", i))?;
    for x in 0..COLUMNS {
        for y in 0..ROWS {
            let found = robots.iter().find(|r| r.position == Coord{x,y}).is_some();
            w.write_fmt(format_args!("{}", if found {'#'} else {'.'}))?;
        }
        w.write("\n".as_bytes())?;
    }

    Ok(())
}

fn run_for(robot: &mut Robot, time: i64) {
    robot.position = Coord {
        x: (robot.position.x + time * robot.velocity.x).rem_euclid(COLUMNS),
        y: (robot.position.y + time * robot.velocity.y).rem_euclid(ROWS),
    }
}

fn count_quadrants(robots: &Vec<Robot>) -> i64 {
    let (mut q1, mut q2, mut q3, mut q4) = (0, 0, 0, 0);
    for robot in robots {
        if robot.position.x < COLUMNS / 2 {
            if robot.position.y < ROWS / 2 {
                q1 += 1
            }
            if robot.position.y > ROWS / 2 {
                q3 += 1
            }
        }
        if robot.position.x > COLUMNS / 2 {
            if robot.position.y < ROWS / 2 {
                q2 += 1
            }
            if robot.position.y > ROWS / 2 {
                q4 += 1
            }
        }
    }
    q1*q2*q3*q4
}

fn parse_input() -> Result<Vec<Robot>> {
    stdin()
        .lines()
        .map(|line| -> Result<Robot> {
            let line = line?;
            let cap = RE
                .captures(&line)
                .context("invalid robot state in line")?;
            Ok(Robot {
                position: Coord{
                    x: cap[1].parse()?,
                    y: cap[2].parse()?,
                },
                velocity: Offset{
                    x: cap[3].parse()?,
                    y: cap[4].parse()?,
                },
            })
        })
        .collect()
}