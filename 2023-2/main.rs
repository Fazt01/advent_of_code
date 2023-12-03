use std::collections::HashMap;
use std::io;
use anyhow::{Context, Error};
use once_cell::sync::Lazy;
use regex::{Regex};

static RE_ID: Lazy<Regex> = Lazy::new(|| Regex::new(r#"Game (\d+)"#).unwrap());

fn main() -> Result<(), Error> {
    Lazy::force(&RE_ID);

    let stdin = io::stdin();
    let limit_map = HashMap::from([
        ("red", 12),
        ("green", 13),
        ("blue", 14),
    ]);

    // // lifetime troubles when trying to use map
    // let sum: i32 = stdin.lines().
    //     map(|line| parse_game_state(&line?)).
    //     // map(|game_state_result| possible_game_id(&game_state_result?, &limit_map)).
    //     // try_fold(0, |acc, game_id|
    //     //     Ok::<i32, Error>(acc + game_id?.unwrap_or(0))
    //     // )?;
    //     map(|game_state_result| Ok(minimum_limit(&game_state_result?.1))).
    //     sum::<Result<i32, Error>>()?;

    let mut sum: i32 = 0;
    for line in stdin.lines() {
        // solved lifetime troubles - line lives for the whole loop iteration
        let line = &line?;

        let game_state = parse_game_state(line)?;

        //part1
        // let game_id = possible_game_id(&game_state, &limit_map)?;
        // match game_id {
        //     Some(game_id) => sum += game_id,
        //     _ => {}
        // }

        //part2
        let min = minimum_limit(&game_state.1);
        sum += min;
    }

    println!("{}", sum);

    Ok(())
}

fn possible_game_id(game_state: &(i32, Vec<HashMap<&str, i32>>), limit_map: &HashMap<&str, i32>) -> Result<Option<i32>, Error> {
    for round_state in &game_state.1 {
        if !is_possible_state(&round_state, limit_map)? {
            return Ok(None);
        }
    }

    Ok(Some(game_state.0))
}

fn parse_game_state(line: &str) -> Result<(i32, Vec<HashMap<&str, i32>>), Error> {
    let (header, state_str) = line.split_once(": ").context("missing line header 'game <id>:'")?;

    let game_id_str = RE_ID.captures(header).context("no line header match")?.extract::<1>().1[0];
    let game_id = game_id_str.parse::<i32>().context("parse game  id")?;

    let mut result = Vec::<HashMap<&str, i32>>::new();
    for round_str in state_str.split("; ") {
        let mut round_state = HashMap::<&str, i32>::new();
        for color_state_str in round_str.split(", ") {
            let (amount_str, color) = color_state_str.split_once(" ").context("no space in color state")?;
            let amount = amount_str.parse::<i32>().context("parse color")?;
            round_state.insert(color, amount);
        }

        result.push(round_state);
    }

    Ok((game_id, result))
}

fn is_possible_state(actual: &HashMap<&str, i32>, limit: &HashMap<&str, i32>) -> Result<bool, Error> {
    for (actual_color, actual_amount) in actual {
        let limit_amount = limit.get(actual_color).context("unknown color")?;
        if limit_amount < actual_amount {
            return Ok(false);
        }
    }

    Ok(true)
}

fn minimum_limit(game_state: &Vec<HashMap<&str, i32>>) -> i32 {
    let mut minimum_state = HashMap::<&str, i32>::new();
    for round_state in game_state {
        for (color, amount) in round_state {
            if amount > minimum_state.get(color).unwrap_or(&0) {
                minimum_state.insert(color, *amount);
            }
        }
    }

    let mut product = 1;
    for (_, amount) in minimum_state {
        product *= amount;
    }

    product
}