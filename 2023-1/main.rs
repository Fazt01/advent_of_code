use std::io;
use std::iter::repeat;
use anyhow::Error;

fn main() -> Result<(), Error> {
    let stdin = io::stdin();
    let mut sum = 0;
    for line in stdin.lines() {
        let line = line?;
        let mut digits: Option<Vec<char>> = None;

        for (i, b) in line.bytes().enumerate() {
            let digit: Option<char>;
            if b >= b'0' && b <=b'9' {
                digit = Some(b.into());
            } else {
                digit = parse_buf(&line.as_bytes().to_vec()[0..=i]);
            }
            if digit.is_some() {
                if digits.is_none() {
                    digits = Some(repeat(digit.unwrap()).take(2).collect());
                }
                digits.as_mut().unwrap()[1] = digit.unwrap();
            }
        }
        let num = String::from_iter(digits.as_ref().unwrap().into_iter()).parse::<i64>()?;
        sum += num;

    }

    println!("{}", sum);

    Ok(())
}

const NUMBERS_STR: [&[u8]; 9] = [b"one", b"two", b"three", b"four", b"five", b"six", b"seven", b"eight", b"nine"];

fn parse_buf(buf: &[u8]) -> Option<char>{
    for (i, number_str) in NUMBERS_STR.iter().enumerate() {
        if buf.len() < number_str.len() {
            continue
        }
        let substr = &buf[&buf.len()-number_str.len()..];
        if substr == *number_str {
            return char::from_digit((i+1) as u32, 10)
        }
    }
    None
}