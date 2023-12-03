use std::collections::HashSet;
use std::fmt::{Debug, Formatter};
use std::io;
use anyhow::Error;

#[derive(Default)]
struct State {
    state: Vec<Vec<u8>>,
    rows: u64,
    columns: u64,
}

impl Debug for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<state>")
    }
}

#[derive(Debug, Copy, Clone)]
enum PointOfView {
    Top,
    Left,
    Bottom,
    Right,
}

static POINT_OF_VIEWS: [PointOfView; 4] = [
    PointOfView::Top,
    PointOfView::Left,
    PointOfView::Bottom,
    PointOfView::Right,
];

#[derive(Debug)]
struct LineOfSight {
    pov: PointOfView,
    tree_line_from_left_idx: u64,
}

impl State {
    fn iter(&self, pov: PointOfView) -> PointOfViewIter {
        PointOfViewIter {
            state: self,
            pov,
            tree_line_from_left_idx: 0,
        }
    }

    // exclusive of target tree
    fn los_iter_from_tree(&self, tree: TreeCoordinates, pov: PointOfView) -> LineOfSightIter {
        let res = LineOfSightIter {
            state: self,
            los: LineOfSight {
                pov,
                tree_line_from_left_idx: match pov {
                    PointOfView::Top => self.columns - 1 - tree.column,
                    PointOfView::Left => tree.row,
                    PointOfView::Bottom => tree.column,
                    PointOfView::Right => self.rows - 1 - tree.row,
                },
            },
            tree_num: match pov {
                PointOfView::Top => tree.row + 1,
                PointOfView::Left => tree.column + 1,
                PointOfView::Bottom => self.rows - tree.row,
                PointOfView::Right => self.columns - tree.column,
            },
        };
        res
    }
}

#[derive(Eq, Hash, PartialEq, Debug)]
struct TreeCoordinates {
    row: u64,
    column: u64,
}

struct Tree {
    coords: TreeCoordinates,
    height: u8,
}

struct PointOfViewIter<'a> {
    state: &'a State,
    pov: PointOfView,
    tree_line_from_left_idx: u64,
}

impl<'a> Iterator for PointOfViewIter<'a> {
    type Item = LineOfSightIter<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match &self.pov {
            PointOfView::Top | PointOfView::Bottom => {
                if self.tree_line_from_left_idx >= self.state.columns {
                    return None;
                }
            }
            PointOfView::Left | PointOfView::Right => {
                if self.tree_line_from_left_idx >= self.state.rows {
                    return None;
                }
            }
        }
        let tree_line_idx = self.tree_line_from_left_idx;
        self.tree_line_from_left_idx += 1;
        Some(LineOfSightIter {
            state: self.state,
            los: LineOfSight {
                pov: self.pov,
                tree_line_from_left_idx: tree_line_idx,
            },
            tree_num: 0,
        })
    }
}

#[derive(Debug)]
struct LineOfSightIter<'a> {
    state: &'a State,
    los: LineOfSight,
    tree_num: u64,
}

impl<'a> Iterator for LineOfSightIter<'a> {
    type Item = Tree;

    fn next(&mut self) -> Option<Self::Item> {
        let column: u64;
        let row: u64;
        match self.los.pov {
            PointOfView::Top => {
                if self.tree_num >= self.state.rows {
                    return None;
                }
                column = self.state.columns - 1 - self.los.tree_line_from_left_idx;
                row = self.tree_num;
            }
            PointOfView::Left => {
                if self.tree_num >= self.state.columns {
                    return None;
                }
                column = self.tree_num;
                row = self.los.tree_line_from_left_idx;
            }
            PointOfView::Bottom => {
                if self.tree_num >= self.state.rows {
                    return None;
                }
                column = self.los.tree_line_from_left_idx;
                row = self.state.rows - 1 - self.tree_num;
            }
            PointOfView::Right => {
                if self.tree_num >= self.state.columns {
                    return None;
                }
                column = self.state.columns - 1 - self.tree_num;
                row = self.state.rows - 1 - self.los.tree_line_from_left_idx;
            }
        }
        self.tree_num += 1;
        Some(Tree {
            coords: TreeCoordinates {
                row,
                column,
            },
            height: self.state.state[row as usize][column as usize],
        })
    }
}

fn main() -> Result<(), Error> {
    let stdin = io::stdin();
    let mut state: State = Default::default();
    for line in stdin.lines() {
        let line = line?; // satisfy borrow checker - live for whole loop iteration
        let line_bytes = line.as_bytes();

        parse_line(line_bytes, &mut state)
    }

    //part 1
    let mut visible_trees = HashSet::<TreeCoordinates>::new();

    for pov in POINT_OF_VIEWS {
        for tree_line in state.iter(pov) {
            let mut visible_line: Option<u8> = None;
            for tree in tree_line {
                if match visible_line {
                    None => true,
                    Some(visible_line) => tree.height > visible_line
                } {
                    visible_trees.insert(tree.coords);
                    visible_line = Some(tree.height)
                }
            }
        }
    }
    println!("{}", visible_trees.len());

    // part 2
    let mut max_visibility = 0;
    for row in 0..state.rows {
        for column in 0..state.columns {
            let tree_height = state.state[row as usize][column as usize];
            let mut visibilities = Vec::<i32>::new();
            for pov in POINT_OF_VIEWS {
                let mut visibility = 0;
                for tree in state.los_iter_from_tree(TreeCoordinates { row, column }, pov) {
                    visibility += 1;
                    if tree.height >= tree_height {
                        break;
                    }
                }
                visibilities.push(visibility);
            }
            let visibility_product = visibilities.iter().product();
            if max_visibility < visibility_product {
                max_visibility = visibility_product
            }
        }
    }
    println!("{}", max_visibility);

    Ok(())
}

fn parse_line(line: &[u8], state: &mut State) {
    let mut row = Vec::<u8>::with_capacity(state.columns as usize);
    for byte in line {
        if *byte >= b'0' && *byte <= b'9' {
            row.push(*byte - b'0')
        }
    }
    state.columns += 1;
    if state.rows == 0 {
        state.rows = row.len() as u64;
    }
    state.state.push(row);
}