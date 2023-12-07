use std::cmp::{Ordering, Reverse};
use std::collections::HashMap;
use std::io;
use anyhow::{Result, Ok, Context};

struct Play {
    hand: Hand,
    bet: u64,
}

#[derive(PartialOrd, PartialEq, Eq)]
struct Hand {
    hand: Vec<Card>,
    type_: HandType,
}

impl Ord for Hand {
    fn cmp(&self, other: &Self) -> Ordering {
        self.type_.cmp(&other.type_).then(self.hand.cmp(&other.hand))
    }
}

impl Hand {
    fn from_str(s: &str, with_jokers: bool) -> Result<Hand> {
        let mut result: Vec<Card> = Vec::new();
        for c in s.chars() {
            result.push(Card::from_char(c, with_jokers));
        }

        let mut card_counts: HashMap<Card, u8> = HashMap::new();
        let mut jokers = 0;
        for card in &result {
            if matches!(card, Card::Joker) {
                jokers += 1;
                continue
            }
            card_counts.insert(*card, *card_counts.get(&card).unwrap_or(&0) + 1);
        }
        let mut card_counts_vec: Vec<(Card, u8)> = card_counts.into_iter().collect();
        card_counts_vec.sort_by_key(|x| Reverse(x.1));
        if card_counts_vec.len() == 0 {
            card_counts_vec.push((Card::Joker, 5))
        } else {
            card_counts_vec[0].1 = card_counts_vec[0].1 + jokers;
        }
        Ok(Hand {
            hand: result,
            type_: match &card_counts_vec[..] {
                [first, ..] if first.1 == 5 => {
                    HandType::FiveOfAKind
                }
                [first, ..] if first.1 == 4 => {
                    HandType::FourOfAKind
                }
                [first, second, ..] if first.1 == 3 && second.1 == 2 => {
                    HandType::FullHouse
                }
                [first, ..] if first.1 == 3 => {
                    HandType::ThreeOfAKind
                }
                [first, second, ..] if first.1 == 2 && second.1 == 2 => {
                    HandType::TwoPair
                }
                [first, ..] if first.1 == 2 => {
                    HandType::OnePair
                }
                _ => HandType::HighCard
            },
        })
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Debug)]
enum HandType {
    HighCard,
    OnePair,
    TwoPair,
    ThreeOfAKind,
    FullHouse,
    FourOfAKind,
    FiveOfAKind,
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Debug)]
enum Card {
    Joker,
    Num(u8),
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

impl Card {
    fn from_char(c: char, with_jokers: bool) -> Card {
        match c {
            'A' => Card::Ace,
            'K' => Card::King,
            'Q' => Card::Queen,
            'J' => match with_jokers {
                true => {Card::Joker}
                false => {Card::Jack}
            }
            'T' => Card::Ten,
            n => Card::Num(n as u8 - '0' as u8),
        }
    }
}

fn main() -> Result<()> {
    // bool parameter part 2
    let mut input = parse(true)?;

    input.sort_by(|v1, v2| v1.hand.cmp(&v2.hand));
    let mut sum = 0;
    for (i, play) in input.iter().enumerate() {
        sum += (i as u64 + 1) * play.bet;
    }

    println!("{}", sum);

    Ok(())
}

fn parse(with_jokers: bool) -> Result<Vec<Play>> {
    let stdin = io::stdin();
    stdin.lines().into_iter().map(|line| {
        let line = line?;
        let split = line.split_once(" ").context("missing space in line")?;
        Ok(Play {
            hand: Hand::from_str(split.0, with_jokers)?,
            bet: split.1.parse()?,
        })
    }).collect()
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use std::cmp::Ordering;
    use crate::Hand;

    #[rstest]
    #[case("K32KK", "3K2KK")]
    #[case("93299", "39299")]
    #[case("TTTTT", "TTT3T")]
    #[case("3TTTT", "TTT33")]
    #[case("3TT3T", "TTTQ3")]
    #[case("TTTQ3", "QQTT3")]
    #[case("QQTT3", "QQTA3")]
    #[case("QQTA3", "Q5TA3")]
    #[case("234A6", "23457")]
    fn no_jokers_ordering_greater(#[case] x: &str, #[case] y: &str) {
        assert_eq!(Hand::from_str(x, false).unwrap().cmp(&Hand::from_str(y, false).unwrap()), Ordering::Greater)
    }

    #[test]
    fn sort_eq() {
        assert_eq!(Hand::from_str("23456", false).unwrap().cmp(&Hand::from_str("23456", false).unwrap()), Ordering::Equal)
    }

    #[test]
    fn sort_last() {
        assert_eq!(Hand::from_str("23456", false).unwrap().cmp(&Hand::from_str("23457", false).unwrap()), Ordering::Less)
    }
}