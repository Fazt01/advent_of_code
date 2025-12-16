use anyhow::{bail, Context, Result};
use itertools::Itertools;
use mathru::algebra::abstr::AbsDiffEq;
use once_cell::sync::Lazy;
use regex::Regex;
use std::cmp::min;
use std::io::stdin;

#[derive(Default)]
struct Machine {
    desired_bulbs: Vec<bool>,
    wirings: Vec<Vec<usize>>,
    joltage: Vec<i64>,
}

fn main() -> Result<()> {
    Lazy::force(&RE);

    let input = parse()?;

    let mut sum1: i64 = 0;

    for machine in &input {
        let mut button_swapped = vec![false; machine.wirings.len()];

        sum1 +=
            minimal_swaps_rec(&mut button_swapped, machine, 0).context("no button combo found")?
    }

    let mut sum2: i64 = 0;

    for machine in &input {
        let desired_joltage = machine.joltage.iter().map(|&x| x as f64).collect_vec();
        let button_limits = machine
            .wirings
            .iter()
            .map(|wiring| {
                wiring
                    .into_iter()
                    .map(|&joltage_i| machine.joltage[joltage_i])
                    .min()
                    .unwrap()
            })
            .collect_vec();

        let mut buttons_matrix =
            Vec::from_iter((0..machine.joltage.len()).map(|_| vec![0f64; machine.wirings.len()]));

        for (button_i, joltages_i) in machine.wirings.iter().enumerate() {
            for &joltage_i in joltages_i {
                buttons_matrix[joltage_i][button_i] = 1f64;
            }
        }

        let result = equations_integer_solve(buttons_matrix, desired_joltage, &button_limits)
            .context("no button combo found")?;

        println!("{:?}", result);

        sum2 += result.iter().sum::<i64>();
    }

    println!("{:?}", sum1);
    println!("{:?}", sum2);

    Ok(())
}

fn minimal_swaps_rec(
    swaps: &mut Vec<bool>,
    machine: &Machine,
    current_index: usize,
) -> Option<i64> {
    if current_index >= swaps.len() {
        let mut bulbs = vec![false; machine.desired_bulbs.len()];
        let mut swap_count = 0;
        for (button_i, wired_to_bulbs) in machine.wirings.iter().enumerate() {
            if !swaps[button_i] {
                continue;
            }
            swap_count += 1;
            for &bulb_i in wired_to_bulbs {
                bulbs[bulb_i] = !bulbs[bulb_i];
            }
        }
        return if bulbs.eq(&machine.desired_bulbs) {
            Some(swap_count)
        } else {
            None
        };
    }
    swaps[current_index] = false;
    let without_swap = minimal_swaps_rec(swaps, machine, current_index + 1);
    swaps[current_index] = true;
    let with_swap = minimal_swaps_rec(swaps, machine, current_index + 1);

    if let Some(without_swap) = without_swap {
        if let Some(with_swap) = with_swap {
            Some(min(without_swap, with_swap))
        } else {
            Some(without_swap)
        }
    } else {
        with_swap
    }
}

static EPS: f64 = 1e-5;

pub fn equations_integer_solve(
    mut a: Vec<Vec<f64>>,
    mut b: Vec<f64>,
    vars_max_bound: &[i64],
) -> Result<Vec<i64>> {
    let rows = a.len();
    let cols = a[0].len();

    let mut row = 0;

    for col in 0..cols {
        let pivot = (row..rows).find(|&r| a[r][col].abs_diff_ne(&0f64, EPS));
        if pivot.is_none() {
            row += 1;
            if row == rows {
                break;
            }
            continue;
        }
        let pivot = pivot.unwrap();

        a.swap(row, pivot);
        b.swap(row, pivot);

        for r in (row + 1)..rows {
            let ratio = a[r][col] / a[row][col];
            for c in col..cols {
                a[r][c] -= ratio * a[row][c];
            }
            b[r] -= ratio * b[row];
        }

        row += 1;
        if row == rows {
            break;
        }
    }

    let mut integer_solutions =
        DependantIntegerSolution::Base(DependantIntegerSolutionBase { size: cols });

    for r in (0..row).rev() {
        integer_solutions =
            DependantIntegerSolution::Dependant(Box::new(DependantIntegerSolutionInner {
                partial_solution_provider: integer_solutions,
                equation_var_coefficients: &a[r],
                equation_result: b[r],
                vars_max_bound,
            }));
    }

    let min = integer_solutions
        .into_iter()
        .min_by_key(|vars| vars.iter().flatten().sum::<i64>());
    let min = min.context("no solution")?;

    for (i, var) in min.iter().enumerate() {
        if var.is_none() {
            bail!("variable {} remained undefined", i);
        }
    }

    Ok(min.iter().map(|x| x.unwrap()).collect())
}

enum DependantIntegerSolution<'a> {
    Dependant(Box<DependantIntegerSolutionInner<'a>>),
    Base(DependantIntegerSolutionBase),
}

impl<'a> IntoIterator for &'a DependantIntegerSolution<'a> {
    type Item = Vec<Option<i64>>;
    type IntoIter = DependantIntegerSolutionIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        DependantIntegerSolutionIter {
            source: self,
            chained: match self {
                DependantIntegerSolution::Dependant(x) => {
                    Some(x.partial_solution_provider.into_iter().into())
                }
                DependantIntegerSolution::Base(_) => None,
            },
            state: None,
            done: false,
        }
    }
}

struct DependantIntegerSolutionBase {
    size: usize,
}

struct DependantIntegerSolutionInner<'a> {
    partial_solution_provider: DependantIntegerSolution<'a>,
    equation_var_coefficients: &'a [f64],
    equation_result: f64,
    vars_max_bound: &'a [i64],
}

struct DependantIntegerSolutionIter<'a> {
    source: &'a DependantIntegerSolution<'a>,
    chained: Option<Box<DependantIntegerSolutionIter<'a>>>,
    state: Option<DependantIntegerSolutionIterState>,
    done: bool,
}

struct DependantIntegerSolutionIterState {
    iterating_vars_i: Vec<usize>,
    iterator_inner: Box<dyn Iterator<Item = Vec<i64>>>,
    chained_last: Vec<Option<i64>>,
}

impl<'a> Iterator for DependantIntegerSolutionIter<'a> {
    type Item = Vec<Option<i64>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        match self.source {
            DependantIntegerSolution::Dependant(source) => {
                if self.state.is_none() {
                    if !self.new_state_from_chained(source) {
                        self.done = true;
                        return None;
                    }
                }

                loop {
                    let state = self.state.as_mut().unwrap();
                    let mut vars = state.chained_last.clone();
                    while let Some(candidate) = state.iterator_inner.next() {
                        for (i, &var_i) in state.iterating_vars_i.iter().enumerate() {
                            vars[var_i] = Some(candidate[i])
                        }
                        let mut sum = 0f64;
                        for i in 0..source.equation_var_coefficients.len() {
                            sum += source.equation_var_coefficients[i]
                                * vars[i].unwrap_or_default() as f64;
                        }
                        if sum.abs_diff_eq(&source.equation_result, EPS) {
                            return Some(vars.clone());
                        }
                    }
                    if !self.new_state_from_chained(source) {
                        self.done = true;
                        return None;
                    }
                }
            }
            DependantIntegerSolution::Base(source) => {
                self.done = true;
                Some(vec![None; source.size])
            }
        }
    }
}

impl<'a> DependantIntegerSolutionIter<'a> {
    fn new_state_from_chained(&mut self, source: &Box<DependantIntegerSolutionInner>) -> bool {
        let chained_next = self.chained.as_mut().unwrap().next();
        if chained_next.is_none() {
            self.done = true;
            return false;
        }
        let chained_last = chained_next.unwrap();

        let iterating_vars_i = source
            .equation_var_coefficients
            .iter()
            .enumerate()
            .filter(|&(i, &v)| v.abs_diff_ne(&0f64, EPS) && chained_last[i].is_none())
            .map(|(i, _)| i)
            .collect_vec();

        let iterating_vars = iterating_vars_i
            .iter()
            .map(|&i| 0..=source.vars_max_bound[i])
            .collect_vec();

        let iterator_inner = Box::new(iterating_vars.iter().cloned().multi_cartesian_product());

        self.state = Some(DependantIntegerSolutionIterState {
            iterating_vars_i,
            iterator_inner,
            chained_last,
        });
        true
    }
}

static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[(.*)] (.*) \{(.*)}").unwrap());

fn parse() -> Result<Vec<Machine>> {
    let mut result = vec![];

    for line in stdin().lines() {
        let mut machine = Machine::default();
        let line = line?;
        let captures = RE.captures(&line).context("invalid input line")?;
        for b in captures[1].bytes() {
            machine.desired_bulbs.push(match b {
                b'#' => true,
                b'.' => false,
                _ => bail!("invalid bulb input {}", &captures[1]),
            })
        }
        for wiring_group_str in captures[2].split(' ') {
            let inner = &wiring_group_str[1..wiring_group_str.len() - 1];
            machine.wirings.push(
                inner
                    .split(',')
                    .map(|s| Ok(s.parse()?))
                    .collect::<Result<Vec<_>>>()?,
            );
        }
        machine.joltage = captures[3]
            .split(',')
            .map(|s| Ok(s.parse()?))
            .collect::<Result<Vec<_>>>()?;
        result.push(machine)
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::equations_integer_solve;
    use rstest::rstest;

    #[rstest]
    #[case(
        vec![
            vec![1f64, 0f64, 2f64, 0f64],
            vec![0f64, 1f64, 0f64, 3f64],
            vec![0f64, 0f64, 0f64, 0f64],
            vec![0f64, 0f64, 0f64, 0f64]
        ],
        vec![11f64, 10f64, 0f64, 0f64],
        vec![1, 1, 5, 3]
    )]
    fn test_equations_integer_solve(
        #[case] matrix: Vec<Vec<f64>>,
        #[case] vector: Vec<f64>,
        #[case] expected: Vec<i64>,
    ) {
        assert_eq!(equations_integer_solve(matrix, vector, &vec![20; 4]).unwrap(), expected)
    }
}
