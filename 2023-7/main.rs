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
    fn from_str(s: &str) -> Result<Hand> {
        let mut result: Vec<Card> = Vec::new();
        for c in s.chars() {
            result.push(Card::from_char(c));
        }

        let mut card_counts: HashMap<Card, u8> = HashMap::new();
        for card in &result {
            card_counts.insert(*card, *card_counts.get(&card).unwrap_or(&0) + 1);
        }
        let mut card_counts_vec: Vec<(Card, u8)> = card_counts.into_iter().collect();
        card_counts_vec.sort_by_key(|x| Reverse(x.1));
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
    Num(u8),
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

impl Card {
    fn from_char(c: char) -> Card {
        match c {
            'A' => Card::Ace,
            'K' => Card::King,
            'Q' => Card::Queen,
            'J' => Card::Jack,
            'T' => Card::Ten,
            n => Card::Num(n as u8 - '0' as u8),
        }
    }
}

fn main() -> Result<()> {
    let mut input = parse()?;

    input.sort_by(|v1, v2| v1.hand.cmp(&v2.hand));
    let mut sum = 0;
    for (i, play) in input.iter().rev().enumerate() {
        println!("{:?} {:?} {}", &play.hand.hand, &play.hand.type_, play.bet);
        sum += (i as u64 + 1) * play.bet
    }

    // part 1
    println!("{}", sum);

    Ok(())
}

fn parse() -> Result<Vec<Play>> {
    let stdin = io::stdin();
    stdin.lines().into_iter().map(|line| {
        let line = line?;
        let split = line.split_once(" ").context("missing space in line")?;
        Ok(Play {
            hand: Hand::from_str(split.0)?,
            bet: split.1.parse()?,
        })
    }).collect()
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;
    use crate::Hand;

    #[test]
    fn same_type() {
        assert_eq!(Hand::from_str("K32KK").unwrap().cmp(&Hand::from_str("3K2KK").unwrap()), Ordering::Greater)
    }

    #[test]
    fn same_type_num() {
        assert_eq!(Hand::from_str("93299").unwrap().cmp(&Hand::from_str("39299").unwrap()), Ordering::Greater)
    }

    #[test]
    fn type_five_over_four() {
        assert_eq!(Hand::from_str("TTTTT").unwrap().cmp(&Hand::from_str("TTT3T").unwrap()), Ordering::Greater)
    }

    #[test]
    fn type_four_over_full_house() {
        assert_eq!(Hand::from_str("3TTTT").unwrap().cmp(&Hand::from_str("TTT33").unwrap()), Ordering::Greater)
    }

    #[test]
    fn type_full_house_over_three() {
        assert_eq!(Hand::from_str("3TT3T").unwrap().cmp(&Hand::from_str("TTTQ3").unwrap()), Ordering::Greater)
    }

    #[test]
    fn type_three_over_two_pair() {
        assert_eq!(Hand::from_str("TTTQ3").unwrap().cmp(&Hand::from_str("QQTT3").unwrap()), Ordering::Greater)
    }

    #[test]
    fn type_two_pair_over_pair() {
        assert_eq!(Hand::from_str("QQTT3").unwrap().cmp(&Hand::from_str("QQTA3").unwrap()), Ordering::Greater)
    }

    #[test]
    fn type_pair_over_high() {
        assert_eq!(Hand::from_str("QQTA3").unwrap().cmp(&Hand::from_str("Q5TA3").unwrap()), Ordering::Greater)
    }
}