use anyhow::{Context, Result};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::io::stdin;
use std::rc::Rc;

type Label = Rc<str>;

type Connections = HashMap<Label, Vec<Label>>;

#[derive(Hash, Eq, PartialEq, Clone)]
struct PossiblePath {
    remaining_intermediate: BTreeSet<Label>,
    end: Label,
}

fn main() -> Result<()> {
    let input = parse()?;

    println!("{}", count_paths(&input, "you", "out", &vec![])?);
    println!("{}", count_paths(&input, "svr", "out", &vec!["fft", "dac"])?);

    Ok(())
}

fn count_paths(
    connections: &Connections,
    start: &str,
    target: &str,
    needed_intermediate: &[&str],
) -> Result<i64> {
    let mut visited: HashMap<PossiblePath, i64> = Default::default();
    let start = Label::from(start);
    visited.insert(
        PossiblePath {
            remaining_intermediate: BTreeSet::from_iter(needed_intermediate.iter().map(|&s| Label::from(s))),
            end: start.clone(),
        },
        1,
    );

    while let Some(new_visited) = expand_visited(connections, &visited, target)? {
        visited = new_visited
    }

    let mut sum = 0;
    for (possible_path, count) in visited {
        if !possible_path.remaining_intermediate.is_empty() {
            continue
        }
        sum += count
    }

    Ok(sum)
}

fn expand_visited(
    connections: &Connections,
    visited: &HashMap<PossiblePath, i64>,
    target: &str,
) -> Result<Option<HashMap<PossiblePath, i64>>> {
    let mut result: HashMap<PossiblePath, i64> = Default::default();

    let mut paths_updated = false;
    for (existing_path, &count) in visited {
        if existing_path.end.as_ref() == target {
            *result.entry(existing_path.clone()).or_default() += count;
            continue;
        }
        paths_updated = true;
        let out_vec = connections
            .get(existing_path.end.as_ref())
            .with_context(|| {
                format!(
                    "output to machine '{}' but no such machine found",
                    existing_path.end
                )
            })?;
        for output in out_vec {
            let mut new_intermediates = existing_path.remaining_intermediate.clone();
            new_intermediates.remove(output);
            let new_path = PossiblePath {
                remaining_intermediate: new_intermediates,
                end: output.clone(),
            };
            *result.entry(new_path).or_default() += count;
        }
    }

    if !paths_updated {
        return Ok(None);
    }

    Ok(Some(result))
}

fn parse() -> Result<Connections> {
    let mut result = Connections::default();
    let mut labels = HashSet::default();

    for line in stdin().lines() {
        let line = line?;
        let mut main_split = line.split(": ");
        let main_label = get_label(
            &mut labels,
            main_split
                .next()
                .context("expected the line main machine label")?,
        );
        let mut output_vec = vec![];
        for output_label in main_split
            .next()
            .context("expected at least one output")?
            .split(" ")
        {
            output_vec.push(get_label(&mut labels, output_label));
        }
        result.insert(main_label, output_vec);
    }

    Ok(result)
}

fn get_label(labels_set: &mut HashSet<Label>, s: &str) -> Rc<str> {
    let present = labels_set.get(s);
    if let Some(present) = present {
        return present.clone();
    }
    let new: Rc<str> = Rc::from(s);
    labels_set.insert(new.clone());
    new
}
