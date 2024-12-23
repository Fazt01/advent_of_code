use anyhow::Result;
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use std::io::stdin;

type Node = String;
type Connection = [Node; 2];
type Clique = Vec<Node>;

fn main() -> Result<()> {
    let input = parse_input()?;

    let mut adjecency: HashMap<Node, HashSet<Node>> = Default::default();

    let mut cliques2: HashSet<Clique> = Default::default();
    for connection in &input {
        adjecency.entry(connection[0].clone()).or_default().insert(connection[1].clone());
        adjecency.entry(connection[1].clone()).or_default().insert(connection[0].clone());
        let mut clique = vec![connection[0].clone(), connection[1].clone()];
        clique.sort();
        cliques2.insert(clique);
    }

    let mut cliques: HashMap<u64, HashSet<Clique>> = HashMap::from([(2, cliques2)]);
    let mut max_clique = None;
    for k in 3.. {
        if cliques[&(k-1)].is_empty() {
            max_clique = Some(cliques[&(k-2)].iter().next().unwrap());
            break;
        }
        println!("{k}");
        let mut cliques_k: HashSet<Clique> = Default::default();
        for clique in &cliques[&(k-1)] {
            'candidates: for candidate in adjecency.get(&clique[0]).unwrap_or(&HashSet::new())  {
                for node in clique.iter().skip(1) {
                    if !adjecency.get(node).unwrap_or(&HashSet::new()).contains(candidate) {
                        continue 'candidates;
                    }
                }
                let mut clique = clique.clone();
                clique.push(candidate.clone());
                clique.sort();
                cliques_k.insert(clique);
            }
        }
        cliques.insert(k, cliques_k);
    }

    let count = cliques[&3].iter().filter(|&x| {
        for node in x {
            if node.starts_with("t") {
                return true
            }
        }
        false
    }).count();

    println!("{count}");
    println!("{}", max_clique.unwrap().join(","));

    Ok(())
}

fn parse_input() -> Result<Vec<Connection>> {
    Ok(stdin()
        .lines()
        .map(|line| -> Result<Connection> {
            let line = line?;
            let mut split = line.split('-');
            Ok([split.next().unwrap().to_owned(), split.next().unwrap().to_owned()])
        })
        .try_collect()?)
}
