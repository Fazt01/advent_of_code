use std::io;
use anyhow::{Result, Ok, Context};

struct Input {
    races: Vec<Race>,
}

struct Race {
    best_time: u64,
    distance: u64,
}

impl Race {
    fn possible_ways_to_beat(&self) -> u64 {
        // function, where wind_up_time is a variable
        // distance = (best_time-wind_up_time) * speed
        // distance = (best_time-wind_up_time) * wind_up_time
        // therefore
        // y = (best_time-x) * x
        // y = -x^2 + best_time*x
        // find where y > distance, so instead just move down by distance and find 0 intersects
        // 0 = -x^2 + best_time*x - distance
        // roots:
        // det = best_time ^ 2 - 4 * (-1) * (-distance)
        // det = best_time ^ 2 - 4*distance
        // x1 = (-best_time + det) / (2*(-1))
        // x2 = (-best_time - det) / (2*(-1))
        let det: f64 = (self.best_time as f64 * self.best_time as f64 - 4_f64 * self.distance as f64).sqrt();
        let x1: f64 = (-(self.best_time as f64) - det) / -2_f64;
        let x2: f64 = (-(self.best_time as f64) + det) / -2_f64;
        let lower = f64::min(x1,x2);
        let upper = f64::max(x1,x2);
        // small eps, To only capture races where I win, not tie. Also due to possible loss of precision
        let lower = (lower + 0.000001).ceil() as u64;
        let upper = (upper - 0.000001).trunc() as u64;
        let upper = u64::min(self.best_time, upper);
        upper - lower + 1
    }

    fn possible_ways_to_beat_slow(&self) -> u64 {
        let mut ways = 0;
        for wind_up_time in 0..=self.best_time {
            let speed = wind_up_time;
            let remaining_time = self.best_time - wind_up_time;
            if remaining_time * speed > self.distance {
                ways += 1;
            }
        }
        ways
    }
}

fn main() -> Result<()> {
    let input = parse()?;

    let mut result = 1;
    for race in input.races {
        result *= race.possible_ways_to_beat()
    }

    // part 1
    println!("{}", result);

    Ok(())
}

fn parse() -> Result<Input> {
    let metrics = vec!["Time", "Distance"];
    let stdin = io::stdin();
    let lines = stdin.lines();
    let mut values = Vec::<Vec<u64>>::new();
    for (i, line) in lines.enumerate() {
        let line = line?;
        let metric = metrics[i];
        let stripped = line.strip_prefix(&[metric, ": "].join("")).context("line header missing")?;
        // part1
        // let values_str = stripped.split_whitespace();
        // let line_values = values_str.map(|x| Ok(x.parse::<u64>()?)).collect::<Result<_>>()?;
        // part2
        let value_str = stripped.replace(" ", "");
        let line_values = vec![value_str.parse::<u64>()?];
        values.push(line_values)
    }
    let mut result_vec: Vec<Race> = Vec::new();
    for i in 0..values[0].len() {
        result_vec.push(Race {
            best_time: values[0][i],
            distance: values[1][i],
        })
    }
    Ok(Input {
        races: result_vec
    })
}
