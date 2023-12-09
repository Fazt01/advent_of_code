use std::io;
use anyhow::{Result, Ok, bail};

#[derive(Clone)]
struct Sequence(Vec<i64>);

impl Sequence {
    fn differences(&self) -> Sequence {
        Sequence(
            self.0.windows(2)
                .map(|x| x[1] - x[0])
                .collect()
        )
    }

    fn differences_vec(&self) -> Result<Vec<Sequence>> {
        let mut sequences: Vec<Sequence> = vec![self.clone()];
        let mut last_sequence = sequences.last().unwrap();

        loop {
            if last_sequence.0.len() == 0 {
                bail!("single nonzero difference, cannot predict the future");
            }
            if last_sequence.0.iter().all(|x| *x == 0) {
                break;
            }
            sequences.push(last_sequence.differences());
            last_sequence = &sequences.last().unwrap();
        }
        Ok(sequences)
    }

    fn predict(&self) -> Result<i64> {
        let sequences = self.differences_vec()?;

        let mut last_difference: i64 = 0;
        for sequence in sequences.iter().rev() {
            last_difference = sequence.0.last().unwrap() + last_difference
        }

        Ok(last_difference)
    }

    fn predict_first(&self) -> Result<i64> {
        let sequences = self.differences_vec()?;

        let mut first_difference: i64 = 0;
        for sequence in sequences.iter().rev() {
            first_difference = sequence.0.first().unwrap() - first_difference
        }

        Ok(first_difference)
    }
}


fn main() -> Result<()> {
    let sequences = parse()?;

    let sum_last_predictions = sequences
        .iter()
        .map(|x| Ok(x.predict()?))
        .sum::<Result<i64>>()?;

    println!("{}", sum_last_predictions);

    let sum_first_predictions = sequences
        .iter()
        .map(|x| Ok(x.predict_first()?))
        .sum::<Result<i64>>()?;

    println!("{}", sum_first_predictions);

    Ok(())
}

fn parse() -> Result<Vec<Sequence>> {
    let stdin = io::stdin();
    let lines = stdin.lines();
    Ok(lines
        .map(|line|
            Ok(
                Sequence(
                    line?.split_whitespace()
                        .map(|num_str| Ok(num_str.parse::<i64>()?))
                        .collect::<Result<_>>()?
                )
            )
        )
        .collect::<Result<_>>()?
    )
}