use std::fmt::{Display, Formatter};
use anyhow::{bail, Result};
use std::io::{stdin, Read};
use itertools::repeat_n;

type CompressedDiskMap = Vec<u8>;
type UncompressedDiskMap = Vec<Option<u32>>;
struct DisplayableDiskMap<'a> (&'a UncompressedDiskMap);


impl<'a> Display for DisplayableDiskMap<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for &item in self.0 {
            let s = item.map(|x| x.to_string());
            let s = match &s {
                None => ".",
                Some(s) => s.as_str(),
            };
            f.write_fmt(format_args!("({s:4})"))?;
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    let input = parse_input()?;

    let mut decompressed = decompress_disk_map(&input);
    defragment_disk2(&mut decompressed);
    let checksum = checksum(&decompressed);
    println!("{checksum}");

    Ok(())
}

fn decompress_disk_map(input: &CompressedDiskMap) -> UncompressedDiskMap {
    let mut is_file = true;
    let mut file_id = 0;
    input.iter().map(|&len| {
        repeat_n(
            {
                let elem = if is_file {
                    Some(file_id as u32)
                } else {
                    None
                };
                if is_file {
                    file_id += 1;
                }
                is_file = !is_file;
                elem
            },
            len as usize)
    }).flatten().collect()
}

fn defragment_disk(input: &mut UncompressedDiskMap) {
    let mut i = 0;
    let mut j = input.len() - 1;
    loop {
        while i < j && input[i].is_some() {
            i += 1
        }
        while i < j && input[j].is_none() {
            j -= 1
        }
        if i < j {
            input.swap(i, j);
            i += 1;
            j -= 1;
        } else {
            break
        }
    }
}

fn defragment_disk2(input: &mut UncompressedDiskMap) {
    let mut j = input.len() - 1;
    loop {
        while 0 < j && input[j].is_none() {
            j -= 1
        }
        if 0 >= j {
            // reached leftmost slot and there is no more files to move
            break
        }
        let file_id = input[j].unwrap();
        let mut file_size = 0;
        while 0 < j && matches!(input[j], Some(x) if x == file_id) {
            file_size += 1;
            j -= 1;
        }
        if 0 >= j {
            // reached leftmost slot and there is the last file, nowhere to move to
            break
        }

        let mut i = 0;

        let mut gap_size = 0;
        loop {
            while i < j && input[i].is_some() {
                i += 1
            }
            if i >= j {
                break
            }
            gap_size = 0;
            while i <= j && input[i].is_none() {
                gap_size += 1;
                i += 1;
            }
            if gap_size >= file_size {
                // sufficient gap, swap file position
                break
            }
            // gap size too small, try another gap for same file
        }
        if gap_size >= file_size {
            for (x, y) in ((i-gap_size)..(i-gap_size+file_size)).zip((j+1)..(j+1+file_size)) {
                input.swap(x,y)
            }
        }
    }
}

fn checksum(input: &UncompressedDiskMap) -> u64 {
    input
        .iter()
        .enumerate()
        .map(|(i, &id)|
            i as u64 * id.unwrap_or_default() as u64
        )
        .sum()
}

fn parse_input() -> Result<Vec<u8>> {
    let mut line = Vec::new();
    stdin().read_to_end(&mut line)?;
    Ok(line
        .iter()
        .map(|&x| {
            if x >= b'0' && x <= b'9' {
                Ok(x - b'0')
            } else {
                bail!("unexpected character")
            }
        })
        .collect::<Result<_>>()?
    )
}