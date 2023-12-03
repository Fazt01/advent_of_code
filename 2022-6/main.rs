use std::collections::HashSet;
use std::io;
use std::io::Read;
use anyhow::Error;
use ringbuffer::{ConstGenericRingBuffer, RingBuffer};


fn main() -> Result<(), Error> {
    const SIZE: usize = 14;
    let mut buf: ConstGenericRingBuffer<char, SIZE> = ConstGenericRingBuffer::new();
    let stdin = io::stdin();
    for (i, byte) in stdin.bytes().enumerate() {
        let c = char::from(byte?);
        buf.push(c);
        let mut set = HashSet::new();
        if buf.len() < SIZE {
            continue
        }
        let mut duplicate = false;
        for c in &buf {
            if !set.insert(c) {
                duplicate = true;
                break
            }
        }
        if !duplicate {
            println!("{}", i+1);
            return Ok(());
        }
    }
    return Err(Error::msg("not found"))
}
