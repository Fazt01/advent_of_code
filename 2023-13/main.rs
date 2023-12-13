use std::io;
use std::ops::Index;
use anyhow::{Result, Ok, bail};

struct Map {
    points: Vec<Point>,
    rows: usize,
    columns: usize,
}

impl Map {
    fn reflections(&self, smudge_count: u64) -> Vec<ReflectionLine> {
        let mut result: Vec<ReflectionLine> = Vec::new();
        for column in 1..self.columns {
            let reflection = ReflectionLine::VerticalOnColumn(column);
            if self.is_reflection(&reflection, smudge_count) {
                result.push(reflection);
            }
        }

        for row in 1..self.rows {
            let reflection = ReflectionLine::HorizontalOnRow(row);
            if self.is_reflection(&reflection, smudge_count) {
                result.push(reflection);
            }
        }

        result
    }

    fn index(&self, x: usize, y: usize) -> &Point {
        self.points.index(y * self.columns + x)
    }
    fn is_reflection(&self, reflection: &ReflectionLine, smudge_count: u64) -> bool {
        let mut found_smudges = 0;
        match reflection {
            ReflectionLine::VerticalOnColumn(column) => {
                'outer: for reflected_column in *column..self.columns {
                    for row in 0..self.rows {
                        let orig = Coord { x: reflected_column as i64, y: row as i64 };
                        let reflected = self.reflect(&orig, reflection);
                        if !self.is_valid(&reflected) {
                            break 'outer;
                        }
                        if self.index(orig.x as usize, orig.y as usize) != self.index(reflected.x as usize, reflected.y as usize) {
                            found_smudges += 1;
                            if found_smudges > smudge_count {
                                return false;
                            }
                        }
                    }
                }
            }
            ReflectionLine::HorizontalOnRow(row) => {
                'outer: for reflected_row in *row..self.rows {
                    for column in 0..self.columns {
                        let orig = Coord { x: column as i64, y: reflected_row as i64 };
                        let reflected = self.reflect(&orig, reflection);
                        if !self.is_valid(&reflected) {
                            break 'outer;
                        }
                        if self.index(orig.x as usize, orig.y as usize) != self.index(reflected.x as usize, reflected.y as usize) {
                            found_smudges += 1;
                            if found_smudges > smudge_count {
                                return false;
                            }
                        }
                    }
                }
            }
        }
        found_smudges == smudge_count
    }

    fn is_valid(&self, coord: &Coord) -> bool {
        coord.x >= 0 && coord.y >= 0 && coord.x < self.columns as i64 && coord.y < self.rows as i64
    }

    fn reflect(&self, coord: &Coord, reflection: &ReflectionLine) -> Coord {
        match reflection {
            ReflectionLine::VerticalOnColumn(column) => {
                Coord {
                    x: 2 * (*column) as i64 - coord.x - 1,
                    y: coord.y,
                }
            }
            ReflectionLine::HorizontalOnRow(row) => {
                Coord {
                    x: coord.x,
                    y: 2 * (*row) as i64 - coord.y - 1,
                }
            }
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
struct Coord {
    x: i64,
    y: i64,
}

#[derive(PartialEq)]
enum Point {
    Ash,
    Rock,
}

enum ReflectionLine {
    VerticalOnColumn(usize),
    HorizontalOnRow(usize),
}

fn main() -> Result<()> {
    let maps = parse()?;

    for smudge_count in [0,1] {
        let mut sum = 0;

        for map in &maps {
            let reflections = map.reflections(smudge_count);
            for reflection in reflections {
                sum += match reflection {
                    ReflectionLine::VerticalOnColumn(column) => column as u64,
                    ReflectionLine::HorizontalOnRow(row) => 100 * row as u64
                }
            }
        }

        println!("{}", sum);

    }

    Ok(())
}

fn parse() -> Result<Vec<Map>> {
    let stdin = io::stdin();
    let mut result: Vec<Map> = Vec::new();
    let mut points: Vec<Point> = Vec::new();
    let mut rows = 0;
    let mut columns = 0;
    for line in stdin.lines() {
        let line = line?;
        if line.is_empty() {
            result.push(Map {
                points,
                rows,
                columns,
            });
            points = Vec::new();
            rows = 0;
            continue;
        }

        columns = line.chars().count();
        rows += 1;

        for c in line.chars() {
            points.push(match c {
                '#' => Point::Rock,
                '.' => Point::Ash,
                _ => bail!("invalid point")
            });
        }
    }
    if !points.is_empty() {
        result.push(Map {
            points,
            rows,
            columns,
        });
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use crate::{Coord, Map, ReflectionLine};

    #[rstest]
    #[case(0, 0, ReflectionLine::VerticalOnColumn(1), 1, 0)]
    #[case(2, 0, ReflectionLine::VerticalOnColumn(5), 7, 0)]
    #[case(0, 2, ReflectionLine::HorizontalOnRow(5), 0, 7)]
    #[case(0, 7, ReflectionLine::HorizontalOnRow(5), 0, 2)]
    fn test_cross_product(#[case] x: i64, #[case] y: i64, #[case] reflect_line: ReflectionLine, #[case] expect_x: i64, #[case] expect_y: i64) {
        assert_eq!(Map {
            points: vec![],
            columns: 0,
            rows: 0,
        }.reflect(&Coord { x, y }, &reflect_line), Coord { x: expect_x, y: expect_y })
    }
}