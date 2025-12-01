use anyhow::{bail, Context, Result};
use std::io::stdin;

struct Dial {
    num_count: i32,
    state: i32,
}

impl Dial {
    fn do_move_with_through_count(&mut self, move_: Move, checked_through_number: i32) -> i32 {
        let orig_state = self.state;
        match move_.direction {
            Direction::Left => self.state -= move_.count,
            Direction::Right => self.state += move_.count,
        };
        let nowrap_trough_count = match (self.state >= checked_through_number
            && orig_state < checked_through_number)
            || (self.state <= checked_through_number && orig_state > checked_through_number)
        {
            true => 1,
            false => 0,
        };
        let wrap_count = self.state.div_euclid(self.num_count).abs();
        self.state = self.state.rem_euclid(self.num_count);
        let final_wrap_trough_count = match (self.state >= checked_through_number
            && matches!(move_.direction, Direction::Right))
            || (self.state <= checked_through_number && matches!(move_.direction, Direction::Left))
        {
            true => {
                if wrap_count > 0 {
                    1
                } else {
                    0
                }
            }
            false => 0,
        };

        let wrap_through_count = if wrap_count > 1 { wrap_count - 1 } else { 0 };
        nowrap_trough_count + wrap_through_count + final_wrap_trough_count
    }
}

#[derive(Copy, Clone)]
enum Direction {
    Left,
    Right,
}

#[derive(Copy, Clone)]
struct Move {
    direction: Direction,
    count: i32,
}

fn main() -> Result<()> {
    let move_iter = parse_input();

    let mut dial = Dial {
        num_count: 100,
        state: 50,
    };

    let mut sum_part_1 = 0;
    let mut sum_part_2 = 0;

    for move_ in move_iter {
        let move_ = move_?;
        sum_part_2 += dial.do_move_with_through_count(move_, 0);
        if dial.state == 0 {
            sum_part_1 += 1;
        }
    }

    println!("{}", sum_part_1);
    println!("{}", sum_part_2);

    Ok(())
}

fn parse_input() -> impl Iterator<Item = Result<Move>> {
    stdin().lines().map(|line| -> Result<Move> {
        let line = line?;
        let (dir_str, num_str) = line
            .split_at_checked(1)
            .context("expected at least 2 chars on line")?;
        let dir = match dir_str {
            "L" => Direction::Left,
            "R" => Direction::Right,
            _ => bail!("expected L or R direction, got {}", dir_str),
        };
        let num = num_str.parse()?;
        return Ok(Move {
            direction: dir,
            count: num,
        });
    })
}
