use std::collections::{HashMap, HashSet};
use std::io;
use anyhow::{Context, Error};

struct Card {
    winning: HashSet<u32>,
    actual: HashSet<u32>,
}

fn main() -> Result<(), Error> {
    // part1
    let mut sum = 0;

    // part2
    let mut copies = HashMap::<usize, u32>::new(); // map card_index: won_copies (excludes original)
    let mut sum2 = 0;

    let stdin = io::stdin();
    for (i, line) in stdin.lines().enumerate() {
        let line = line?;

        let card = parse_line(&line).with_context(|| format!("parsing line {}", i + 1))?;

        // part 1
        let winning_count = card.actual.intersection(&card.winning).count() as u32;
        let score = if winning_count > 0 { 2_u32.pow(winning_count - 1) } else { 0 };
        sum += score;

        let current_card_instances = *copies.get(&i).unwrap_or(&0) + 1;
        sum2 += current_card_instances;

        for won_copy_index in i + 1..i + 1 + winning_count as usize {
            let prev_copies = copies.get(&won_copy_index).unwrap_or(&0);
            copies.insert(won_copy_index, prev_copies + current_card_instances);
        }
    }

    // part 1
    println!("{}", sum);
    // part 2
    println!("{}", sum2);

    Ok(())
}

fn parse_line(line: &str) -> Result<Card, Error> {
    let (_, state_str) = line.split_once(": ").context("missing line header")?;
    let (winning_str, actual_str) = state_str.split_once(" | ").context("missing winning/actual separator")?;

    fn str_to_card_set(s: &str) -> Result<HashSet<u32>, Error> {
        Ok(s.split_whitespace().
            map(|x| Ok::<_, Error>(x.parse::<u32>()?)).
            collect::<Result<_, _>>()?
        )
    }

    Ok(Card {
        winning: str_to_card_set(winning_str).context("parsing winning card set")?,
        actual: str_to_card_set(actual_str).context("parsing actual card set")?,
    })
}