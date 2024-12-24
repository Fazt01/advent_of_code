use anyhow::{bail, Result};
use itertools::Itertools;
use std::collections::HashMap;
use std::io::stdin;

type Node = String;

struct Puzzle {
    inputs: HashMap<Node, bool>,
    connections: HashMap<Node, Connection>
}

struct Connection {
    a: Node,
    b: Node,
    operator: Operator,
}

#[derive(Debug)]
enum Operator {
    And,
    Or,
    Xor,
}

fn main() -> Result<()> {
    let input = parse_input()?;

    // part 1
    let mut values: HashMap<&str, Option<bool>> = Default::default();
    for (node, &value) in &input.inputs {
        values.insert(node, Some(value));
    }

    for (node, _) in &input.connections {
        values.insert(node, None);
    }

    let mut result_bits = vec![];
    for (node, _) in &input.connections {
        if !node.starts_with("z") {
            continue
        }
        result_bits.push((node, evaluate(node, &mut values, &input.connections)));
    }

    result_bits.sort_by_key(|x| x.0);
    result_bits.reverse();

    let result = to_decimal(result_bits.iter().map(|x| x.1).collect_vec());

    println!("{result}");

    // part 2 - manually inspect the long chain of slow binary adder in graphviz (:

    for (node, connection) in &input.connections {
        println!("{node} [label=\"{node} {:?}\"{}]", connection.operator, if node.starts_with("z") {" color=\"red\""} else {""});
        println!("{} -> {node}", connection.a);
        println!("{} -> {node}", connection.b);
    }

    Ok(())
}

fn to_decimal(result_bits: Vec<bool>) -> u64 {
    let mut result = 0;
    for value in result_bits {
        result *= 2;
        if value {
            result += 1
        }
    }
    result
}

fn evaluate<'a>(node: &'a Node, values: &mut HashMap<&'a str, Option<bool>>, connections: &'a HashMap<String, Connection>) -> bool {
    if let Some(value) = values[node.as_str()] {
        return value;
    }
    let connection = &connections[node];
    let a = evaluate(&connection.a, values, connections);
    let b = evaluate(&connection.b, values, connections);
    let value = match connection.operator {
        Operator::And => a && b,
        Operator::Or => a || b,
        Operator::Xor => a != b,
    };
    values.insert(node, Some(value));
    value
}

fn parse_input() -> Result<Puzzle> {
    let mut lines = stdin().lines();
    let mut inputs: HashMap<Node, bool> = Default::default();
    for line in (&mut lines).take_while(|line| matches!(line, Ok(line) if !line.is_empty())) {
        let line = line?;
        let split = line.split(": ").collect_vec();
        inputs.insert(split[0].to_string(), if split[1] == "1" {true} else {false});
    }

    let mut connections: HashMap<Node, Connection> = Default::default();
    for line in lines {
        let line = line?;
        let split = line.split(" ").collect_vec();
        connections.insert(split[4].to_string(), Connection{
            a: split[0].to_string(),
            b: split[2].to_string(),
            operator: match split[1] {
                "AND" => Operator::And,
                "OR" => Operator::Or,
                "XOR" => Operator::Xor,
                _ => bail!("unexpected operator"),
            },
        });
    }

    Ok(Puzzle{
        inputs,
        connections,
    })
}
