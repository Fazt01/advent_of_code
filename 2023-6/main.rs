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
