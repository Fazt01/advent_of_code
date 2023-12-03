use std::io;
use anyhow::Error;

fn main() -> Result<(), anyhow::Error> {
    let stdin = io::stdin();
    let mut sum = 0;
    for line in stdin.lines() {
        let line = line?;
        let pair = parse_pair(&line)?;
        let range1 = parse_range(pair.0)?;
        let range2 = parse_range(pair.1)?;
        // if (range1.0 >= range2.0 && range1.1 <= range2.1)
        //     || (range2.0 >= range1.0 && range2.1 <= range1.1) {

        if (range1.0 >= range2.0 && range1.0 <= range2.1)
             || (range1.1 >= range2.0 && range1.1 <= range2.1)
             || (range2.0 >= range1.0 && range2.0 <= range1.1)
             || (range2.1 >= range1.0 && range2.1 <= range1.1) {
            sum += 1;
        }
    }
    println!("{}", sum);

    Ok(())
}

fn parse_pair(s: &str) -> Result<(&str, &str), Error> {
    let res: Vec<&str> = s.split(',').collect();
    if res.len() != 2 {
        return Err(Error::msg("not 1 comma"))
    }

    return Ok((res[0], res[1]))
}

fn parse_range(s: &str) -> Result<(i64, i64), Error> {
    let res: Vec<&str> = s.split('-').collect();
    if res.len() != 2 {
        return Err(Error::msg("not 1 comma"))
    }

    return Ok((res[0].parse::<i64>()?, res[1].parse::<i64>()?))
}