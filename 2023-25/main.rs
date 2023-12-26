use std::collections::{HashMap, HashSet};
use std::io;
use std::rc::Rc;
use anyhow::{Result, Ok, Context};
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;

type Graph = petgraph::Graph<NodeInfo, i64, petgraph::Undirected>;

#[derive(Debug)]
struct NodeInfo {
    names: HashSet<Rc<str>>,
}

fn main() -> Result<()> {
    let mut graph = parse()?;

    let orig_node_count = graph.node_count();
    while graph.node_count() >= 2 {
        // Stoer-Wagner min-cut algorithm
        let (last_edge_weight, node_s, node_t) = cut_phase(&graph);
        let last_node_cardinality = graph.node_weight(node_t).unwrap().names.len();
        if last_edge_weight == 3 {
            println!("{}", last_node_cardinality * (orig_node_count - last_node_cardinality));
            break;
        }
        merge_last_nodes(&mut graph, node_s, node_t);
    }

    Ok(())
}

fn merge_last_nodes(graph: &mut Graph, mut node_s: NodeIndex, node_t: NodeIndex) {
    let mut new_edges = vec![];
    let mut modified_edges = vec![];

    for node_idx in graph.node_indices() {
        for edge in graph.edges_connecting(node_idx, node_t) {
            match graph.edges_connecting(node_idx, node_s).next() {
                None => new_edges.push((node_idx, node_s, *edge.weight())),
                Some(existing_edge) => modified_edges.push((existing_edge.id(), *existing_edge.weight() + *edge.weight())),
            }
        }
    }

    for new_edge in new_edges {
        graph.add_edge(new_edge.0, new_edge.1, new_edge.2);
    }
    for modified_edge in modified_edges {
        *graph.edge_weight_mut(modified_edge.0).unwrap() = modified_edge.1;
    }

    let removed = graph.remove_node(node_t).unwrap();
    if node_s.index() >= graph.node_count() {
        // s was last, and was moved to place of t when t was removed
        node_s = node_t
    }
    let modified = &mut graph.node_weight_mut(node_s).unwrap().names;
    *modified = modified.union(&removed.names)
        .cloned()
        .collect();
}

fn cut_phase(graph: &Graph) -> (i64, NodeIndex, NodeIndex) {
    let initial_node = NodeIndex::new(0);
    let mut in_group: HashSet<NodeIndex> = [initial_node].into();

    let mut node_s = None;
    let mut node_t = Some(initial_node);

    let mut most_connected = Some(initial_node);
    let mut edges_to_node = HashMap::<NodeIndex, i64>::new();

    while graph.node_count() != in_group.len() {
        for edge in graph.edges(most_connected.unwrap()) {
            if in_group.contains(&edge.target()) {
                continue
            }
            *edges_to_node.entry(edge.target()).or_default() += edge.weight();
        }

        edges_to_node.remove(&most_connected.unwrap());

        most_connected = edges_to_node.iter()
            .max_by_key(|(_, &weight)| weight)
            .map(|x| *x.0);
        in_group.insert(most_connected.unwrap());
        node_s = node_t;
        node_t = most_connected;
    }

    let node_s = node_s.unwrap();
    let node_t = node_t.unwrap();

    let cut_off_weight = graph.node_indices().map(|node_idx| {
        if node_idx == node_t {
            return 0
        }
        graph.edges_connecting(node_idx, node_t).map(|x| *x.weight()).sum()
    }).sum();

    (cut_off_weight, node_s, node_t)
}

fn parse() -> Result<Graph> {
    let stdin = io::stdin();
    let mut graph: Graph = Graph::new_undirected();
    let mut node_indexes = HashMap::<Rc<str>, NodeIndex>::new();
    for line in stdin.lines() {
        let line = line?;
        let (from_node, to_nodes) = line.split_once(": ").context("missing : in line")?;
        let to_nodes = to_nodes.split_whitespace().collect::<Vec<_>>();
        let mut ensure_node = |node: &str, from: Option<NodeIndex>| -> NodeIndex {
            let idx = match node_indexes.get(node) {
                None => {
                    let node_ptr = Rc::<str>::from(node);
                    let node_info = NodeInfo { names: [node_ptr.clone()].into() };
                    let idx = graph.add_node(node_info);
                    node_indexes.insert(node_ptr, idx);
                    idx
                }
                Some(&idx) => {
                    idx
                }
            };
            if let Some(from_index) = from {
                graph.add_edge(from_index, idx, 1);
            }
            idx
        };
        let from_index = ensure_node(from_node, None);
        for to_node in to_nodes {
            ensure_node(to_node, Some(from_index));
        }
    }
    Ok(graph)
}