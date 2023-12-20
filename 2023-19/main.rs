use std::cmp::{max, min};
use std::collections::HashMap;
use std::io;
use std::ops::RangeInclusive;
use anyhow::{Result, Ok, Context, bail};
use once_cell::sync::Lazy;
use regex::Regex;

struct Puzzle {
    workflows: HashMap<String, Workflow>,
    objects: Vec<Object>,
}

struct Workflow {
    rules: Vec<Rule>,
}

enum Rule {
    CompareRule(CompareRule),
    AlwaysRule(RuleTarget),
}

#[derive(Clone)]
enum RuleTarget {
    Accept,
    Reject,
    ToWorkflow(String),
}

#[derive(Debug)]
enum ObjectResult {
    Accept,
    Reject,
}

#[derive(Debug)]
struct MultiObjectResult {
    multi_object: MultiObject,
    result: ObjectResult,
}

struct MultiObjectInterimResult {
    multi_object: MultiObject,
    target: RuleTarget,
}


struct CompareRule {
    attribute: ObjectAttribute,
    op: RuleOp,
    cmp_num: i64,
    target: RuleTarget,
}

enum RuleOp {
    LT,
    GT,
}

#[derive(Clone)]
struct Object {
    attributes: HashMap<ObjectAttribute, i64>,
}

#[derive(Clone, Debug)]
struct MultiObject {
    attributes: HashMap<ObjectAttribute, RangeInclusive<i64>>,
}

#[derive(Eq, Hash, PartialEq, Copy, Clone, Debug)]
enum ObjectAttribute {
    X,
    M,
    A,
    S,
}

fn main() -> Result<()> {
    Lazy::force(&RE_OBJECT);
    Lazy::force(&RE_RULE);

    let puzzle = parse()?;

    // part 1
    let mut sum = 0;
    for object in &puzzle.objects {
        if matches!(process_object(&puzzle.workflows, object)?, ObjectResult::Accept) {
            sum += object.attributes.values().sum::<i64>();
        }
    }
    println!("{sum}");

    // part 2
    let all_possibilities_multi_object = MultiObject {
        attributes: [
            ObjectAttribute::X,
            ObjectAttribute::M,
            ObjectAttribute::A,
            ObjectAttribute::S,
        ]
            .into_iter()
            .map(|k| (k, 1..=4000))
            .collect(),
    };

    let multi_object_results = process_multi_object(&puzzle.workflows, all_possibilities_multi_object)?;
    let mut sum = 0;
    for multi_object_result in &multi_object_results {
        if let ObjectResult::Accept = multi_object_result.result {
            sum += multi_object_result.multi_object.attributes
                .values()
                .map(|x| x.clone().count() as i64)
                .product::<i64>();
        }
    }
    println!("{sum}");

    Ok(())
}

fn process_object(workflows: &HashMap<String, Workflow>, object: &Object) -> Result<ObjectResult> {
    let mut current = "in";
    loop {
        let workflow = workflows.get(current).with_context(|| format!("no workflow named \"{current}\""))?;
        for rule in &workflow.rules {
            let target = match rule {
                Rule::CompareRule(compare) => {
                    let attribute_value = *object.attributes.get(&compare.attribute).context("missing object attribute")?;
                    let matched_rule = match compare.op {
                        RuleOp::LT => {
                            attribute_value < compare.cmp_num
                        }
                        RuleOp::GT => {
                            attribute_value > compare.cmp_num
                        }
                    };
                    if !matched_rule {
                        continue;
                    }
                    &compare.target
                }
                Rule::AlwaysRule(target) => {
                    target
                }
            };
            match target {
                RuleTarget::Accept => {
                    return Ok(ObjectResult::Accept);
                }
                RuleTarget::Reject => {
                    return Ok(ObjectResult::Reject);
                }
                RuleTarget::ToWorkflow(str) => {
                    current = str.as_str();
                    break;
                }
            }
        }
    }
}

fn process_multi_object(workflows: &HashMap<String, Workflow>, multi_object: MultiObject) -> Result<Vec<MultiObjectResult>> {
    let current = "in";
    let interim_result = process_multi_object_rec(workflows, multi_object, current, 0)?;
    interim_result
        .into_iter()
        .map(|x| {
            let result = match x.target {
                RuleTarget::Accept => ObjectResult::Accept,
                RuleTarget::Reject => ObjectResult::Reject,
                RuleTarget::ToWorkflow(_) => bail!("not all multi_objects are resolved")
            };
            Ok(MultiObjectResult {
                multi_object: x.multi_object,
                result,
            })
        }).collect::<Result<_>>()
}

fn process_multi_object_rec(workflows: &HashMap<String, Workflow>, multi_object: MultiObject, current_workflow: &str, rule_i: usize) -> Result<Vec<MultiObjectInterimResult>> {
    let workflow = workflows.get(current_workflow).with_context(|| format!("no workflow named \"{current_workflow}\""))?;
    let rule = &workflow.rules[rule_i];
    let mut result = vec![];
    let mut target_paths = vec![];
    match rule {
        Rule::CompareRule(compare) => {
            let attribute_value_range = multi_object.attributes.get(&compare.attribute).context("missing object attribute")?.clone();
            let (left_branch, right_branch) = split_multi_object(
                multi_object,
                compare.attribute,
                &attribute_value_range,
                compare.cmp_num + if matches!(compare.op, RuleOp::GT) { 1 } else { 0 },
            );
            match compare.op {
                RuleOp::LT => {
                    if let Some(left_branch) = left_branch {
                        target_paths.push((left_branch, &compare.target))
                    }
                    if let Some(right_branch) = right_branch {
                        let mut right_branch_results = process_multi_object_rec(
                            workflows, right_branch, current_workflow, rule_i + 1,
                        )?;
                        result.append(&mut right_branch_results);
                    }
                }
                RuleOp::GT => {
                    if let Some(left_branch) = left_branch {
                        let mut right_branch_results = process_multi_object_rec(
                            workflows, left_branch, current_workflow, rule_i + 1,
                        )?;
                        result.append(&mut right_branch_results);
                    }
                    if let Some(right_branch) = right_branch {
                        target_paths.push((right_branch, &compare.target))
                    }
                }
            };
        }
        Rule::AlwaysRule(target) => {
            target_paths.push((multi_object, target));
        }
    };
    for (multi_object, target) in target_paths {
        match target {
            RuleTarget::Accept | RuleTarget::Reject => {
                result.push(MultiObjectInterimResult {
                    multi_object,
                    target: target.clone(),
                });
            }
            RuleTarget::ToWorkflow(s) => {
                let mut target_result = process_multi_object_rec(
                    workflows, multi_object, s.as_str(), 0,
                )?;
                result.append(&mut target_result);
            }
        }
    };
    Ok(result)
}

fn split_multi_object(
    orig: MultiObject,
    object_attribute: ObjectAttribute,
    attribute_value_range: &RangeInclusive<i64>,
    split_on: i64,
) -> (Option<MultiObject>, Option<MultiObject>) {
    let left_range = *attribute_value_range.start()..=min(*attribute_value_range.end(), split_on - 1);
    let right_range = max(*attribute_value_range.start(), split_on)..=*attribute_value_range.end();
    let left = match left_range.is_empty() {
        true => {
            None
        }
        false => {
            let mut clone = orig.clone();
            clone.attributes.insert(object_attribute, left_range);
            Some(clone)
        }
    };
    let right = match right_range.is_empty() {
        true => {
            None
        }
        false => {
            let mut clone = orig.clone();
            clone.attributes.insert(object_attribute, right_range);
            Some(clone)
        }
    };
    (left, right)
}

static RE_OBJECT: Lazy<Regex> = Lazy::new(|| Regex::new(r"\{x=(\d+),m=(\d+),a=(\d+),s=(\d+)}").unwrap());
static RE_RULE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\w)([<>])(\d+):(\w+)").unwrap());

fn parse() -> Result<Puzzle> {
    let stdin = io::stdin();
    let mut puzzle = Puzzle {
        workflows: Default::default(),
        objects: vec![],
    };
    let mut parsing_objects = false;
    for line in stdin.lines() {
        let line = line?;
        match parsing_objects {
            false => {
                if line.is_empty() {
                    parsing_objects = true;
                    continue;
                }
                let (name, workflow) = line.split_once('{').context("missing workflow begin brace")?;
                let workflow = workflow.strip_suffix('}').context("missing closing brace")?;
                let rules = workflow.split(',');
                let rules = rules
                    .map(|s| {
                        if !s.contains(':') {
                            return Ok(Rule::AlwaysRule(parse_rule_target(s)));
                        }
                        let (_, captured) = RE_RULE.captures(s)
                            .with_context(|| format!("invalid rule '{s}'"))?
                            .extract::<4>();
                        Ok(Rule::CompareRule(CompareRule {
                            attribute: match captured[0] {
                                "x" => ObjectAttribute::X,
                                "m" => ObjectAttribute::M,
                                "a" => ObjectAttribute::A,
                                "s" => ObjectAttribute::S,
                                _ => bail!("invalid object {}", captured[0])
                            },
                            op: match captured[1] {
                                "<" => RuleOp::LT,
                                ">" => RuleOp::GT,
                                _ => bail!("invalid comparison {}", captured[1]),
                            },
                            cmp_num: captured[2].parse::<i64>()?,
                            target: parse_rule_target(captured[3]),
                        }))
                    })
                    .collect::<Result<Vec<_>>>()?;
                puzzle.workflows.insert(name.to_owned(), Workflow { rules });
            }
            true => {
                let (_, captured) = RE_OBJECT.captures(line.as_str()).context("invalid object definition")?.extract::<4>();
                puzzle.objects.push(Object {
                    attributes: [
                        (ObjectAttribute::X, captured[0].parse::<i64>()?),
                        (ObjectAttribute::M, captured[1].parse::<i64>()?),
                        (ObjectAttribute::A, captured[2].parse::<i64>()?),
                        (ObjectAttribute::S, captured[3].parse::<i64>()?),
                    ].into(),
                })
            }
        }
    }
    Ok(puzzle)
}

fn parse_rule_target(s: &str) -> RuleTarget {
    match s {
        "A" => RuleTarget::Accept,
        "R" => RuleTarget::Reject,
        s => RuleTarget::ToWorkflow(s.to_owned())
    }
}