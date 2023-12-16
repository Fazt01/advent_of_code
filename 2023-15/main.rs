use std::io;
use anyhow::{Result, Ok, Context};

struct LensBoxes {
    boxes: Vec<LensBox>,
}

impl LensBoxes {
    fn add_lens(&mut self, box_i: u8, label: &str, focal_length: u8) {
        self.boxes[box_i as usize].add_lens(label, focal_length)
    }

    fn remove_lens(&mut self, box_i: u8, label: &str) {
        self.boxes[box_i as usize].remove_lens(label)
    }

    fn focusing_power(&self) -> u64 {
        let mut sum: u64 = 0;
        for (box_i, lens_box) in self.boxes.iter().enumerate() {
            for (lens_i, lens) in lens_box.lenses.iter().enumerate() {
                sum += (box_i as u64 + 1) * (lens_i as u64 + 1) * lens.focal_length as u64;
            }
        }
        sum
    }
}

impl Default for LensBoxes {
    fn default() -> Self {
        LensBoxes {
            boxes: [(); 256].iter().map(|_| Default::default()).collect(),
        }
    }
}

#[derive(Default)]
struct LensBox {
    lenses: Vec<Lens>,
}

impl LensBox {
    fn add_lens(&mut self, label: &str, focal_length: u8) {
        let lens = Lens {
            label: label.to_string(),
            focal_length,
        };
        let pos = self.lenses.iter().position(|x| x.label == label);
        match pos {
            None => {
                self.lenses.push(lens)
            }
            Some(pos) => {
                *self.lenses.get_mut(pos).unwrap() = lens;
            }
        }
    }

    fn remove_lens(&mut self, label: &str) {
        let pos = self.lenses.iter().position(|x| x.label == label);
        let Some(pos) = pos else { return; };
        self.lenses.remove(pos);
    }
}

struct Lens {
    label: String,
    focal_length: u8,
}

fn hash_bytes(bytes: &[u8]) -> u8 {
    let mut current_value: u8 = 0;
    for byte in bytes {
        current_value = ((current_value as u32 + *byte as u32) * 17) as u8;
    }
    current_value
}

fn main() -> Result<()> {
    let stdin = io::stdin();
    let line = stdin.lines().next().context("line missing")??;
    let splits = line.split(",");
    let mut lense_boxes: LensBoxes = Default::default();
    let mut sum: u64 = 0;
    for split in splits {
        let pos = split.find(&['-', '=']).context("operation separator missing")?;
        let label = &split[0..pos];
        let operation = split.as_bytes()[pos];
        let box_i = hash_bytes(label.as_bytes());
        match operation {
            b'-' => {
                lense_boxes.remove_lens(box_i, label)
            }
            b'=' => {
                let focal_length = split.get(pos + 1..).context("missing focal length")?;
                let focal_length = focal_length.parse::<u8>()?;
                lense_boxes.add_lens(box_i, label, focal_length)
            }
            _ => unreachable!("only position of - or = could be found")
        }

        let hash = hash_bytes(split.as_bytes());
        sum += hash as u64;
    }

    println!("{}", sum);
    println!("{}", lense_boxes.focusing_power());

    Ok(())
}
