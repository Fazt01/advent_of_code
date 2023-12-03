use std::collections::hash_map::{Values};
use std::collections::HashMap;
use std::io;
use std::ptr::NonNull;
use anyhow::{Context, Error};

#[derive(Default, Debug)]
struct FileSystem {
    root: Node,
}

#[derive(Debug)]
struct File {
    size: u64,
}

#[derive(Default, Debug)]
struct Dict {
    nested_size: Option<u64>,
    content: HashMap<String, Node>,
}

#[derive(Debug)]
struct Node {
    // always points to the key of hashmap this Node is the value of
    // and to constant empty string for root
    name: NonNull<str>,
    type_: NodeType,
    parent: Option<NonNull<Node>>,
}

impl Default for Node {
    fn default() -> Self {
        Node {
            name: "".into(),
            type_: Default::default(),
            parent: None,
        }
    }
}

#[derive(Debug)]
enum NodeType {
    File(File),
    Dict(Dict),
}

impl Default for NodeType {
    fn default() -> Self { NodeType::Dict(Default::default()) }
}

#[derive(Debug)]
struct State {
    current_directory: NonNull<Node>,
    executing: StateType,
}

#[derive(Debug)]
enum StateType {
    None,
    ListDirectory,
}

#[derive(Debug)]
enum Command<'a> {
    ListDirectory,
    ChangeDirectory(ChangeDirectoryTarget<'a>),
}

#[derive(Debug)]
enum ChangeDirectoryTarget<'a> {
    Root,
    Parent,
    Child(&'a str),
}

#[derive(Debug)]
enum ParsedLine<'a> {
    Command(Command<'a>),
    Listing(ParsedLineListing<'a>),
}

#[derive(Debug)]
enum ParsedLineListing<'a> {
    FileListing(FileListing<'a>),
    DictListing(&'a str),
}

#[derive(Debug)]
struct FileListing<'a> {
    name: &'a str,
    size: u64,
}

fn main() -> Result<(), Error> {
    let stdin = io::stdin();

    let mut file_system: FileSystem = Default::default();
    let mut state: State = State {
        current_directory: NonNull::new(&mut file_system.root).context("null ptr when initializing current directory")?,
        executing: StateType::None,
    };
    for line in stdin.lines() {
        let line = line?;

        let parse_line = parse_line(&line)?;

        // println!("line: {:?}", parse_line);
        execute(&parse_line, &mut state, &mut file_system)?;

        // println!("state: {:?}", state);
        // println!("fs: {:?}", file_system);
    }

    let max_used_size = 40_000_000; // inclusive
    let root_size = calculate_containing_size(&mut file_system.root)?;
    if root_size <= max_used_size {
        return Err(Error::msg("already enough space"));
    }
    let need_to_free_up = root_size - max_used_size;

    //part1
    // let mut sum = 0;
    // for node in find(&file_system.root, |node| {
    //     match &node.type_ {
    //         NodeType::Dict(dict) => {
    //             dict.nested_size.unwrap() <= 100000
    //         }
    //         _ => false
    //     }
    // }) {
    //     match &node.type_ {
    //         NodeType::Dict(dict) => {
    //             sum += dict.nested_size.unwrap()
    //         }
    //         _ => {}
    //     }
    // }
    // println!("{}", sum);

    //part2
    let mut min: Option<u64> = None;
    for node in find(&file_system.root, |node| {
        match &node.type_ {
            NodeType::Dict(dict) => {
                let dict_size = dict.nested_size.unwrap();
                dict_size >= need_to_free_up
            }
            _ => false
        }
    }) {
        let NodeType::Dict(dict) = &node.type_ else { unreachable!() };
        match min {
            None => {
                min = Some(dict.nested_size.context("uninitialized dict size")?)
            }
            Some(previous_min) => {
                let dict_size = dict.nested_size.context("uninitialized dict size")?;
                if previous_min > dict_size {
                    min = Some(dict_size)
                }
            }
        }
    }
    println!("{}", min.context("no suitable directory found")?);

    Ok(())
}

fn find<F>(root: &Node, predicate: F) -> NodeIter<F>
    where
        F: Fn(&Node) -> bool
{
    NodeIter {
        root: Some(root),
        iterator_stack: Vec::new(),
        predicate,
    }
}

fn calculate_containing_size(node: &mut Node) -> Result<u64, Error> {
    match &mut node.type_ {
        NodeType::File(file) => {
            Ok(file.size)
        }
        NodeType::Dict(ref mut dict) => {
            let mut sum = 0;
            for (_, child_node) in dict.content.iter_mut() {
                sum += calculate_containing_size(child_node)?
            }
            dict.nested_size = Some(sum);
            Ok(sum)
        }
    }
}

fn execute(parsed_line: &ParsedLine, state: &mut State, file_system: &mut FileSystem) -> Result<(), Error> {
    match parsed_line {
        ParsedLine::Command(command) => {
            state.executing = StateType::None;
            match command {
                Command::ListDirectory => {
                    state.executing = StateType::ListDirectory
                }
                Command::ChangeDirectory(chdir_target) => {
                    match chdir_target {
                        ChangeDirectoryTarget::Root => {
                            state.current_directory = NonNull::from(&file_system.root);
                        }
                        ChangeDirectoryTarget::Parent => unsafe {
                            state.current_directory = state.current_directory.as_ref().parent.context("attempt to cd parent of root")?;
                        }
                        ChangeDirectoryTarget::Child(child_target) => {
                            let current_directory: &mut Node;
                            unsafe {
                                current_directory = state.current_directory.as_mut();
                            }
                            match &mut current_directory.type_ {
                                NodeType::Dict(ref mut current_dict) => {
                                    let content = &mut current_dict.content;
                                    match content.get_mut(*child_target) {
                                        None => {
                                            let child_target_string = child_target.to_string();
                                            let node = Node {
                                                name: child_target_string.as_str().into(),
                                                type_: NodeType::Dict(Default::default()),
                                                parent: Some(state.current_directory),
                                            };
                                            content.insert(child_target_string, node);
                                            state.current_directory = content.get_mut(*child_target).unwrap().into();
                                        }
                                        Some(child_node) => {
                                            match &mut child_node.type_ {
                                                NodeType::Dict(_) => {
                                                    state.current_directory = child_node.into();
                                                }
                                                _ => {
                                                    return Err(Error::msg("attempt to cd to file that is not dict"));
                                                }
                                            }
                                        }
                                    }
                                }
                                _ => {
                                    return Err(Error::msg("I am in file?"));
                                }
                            }
                        }
                    }
                }
            }
        }
        ParsedLine::Listing(listing) => {
            let current_directory: &mut Node;
            unsafe {
                current_directory = state.current_directory.as_mut();
            }
            match current_directory.type_ {
                NodeType::Dict(ref mut dict) => {
                    let content = &mut dict.content;
                    match listing {
                        ParsedLineListing::FileListing(file_listing) => {
                            let name = file_listing.name.to_string();
                            let node = Node {
                                name: name.as_str().into(),
                                type_: NodeType::File(File {
                                    size: file_listing.size
                                }),
                                parent: Some(state.current_directory),
                            };
                            content.insert(name, node);
                        }
                        ParsedLineListing::DictListing(dict_listing) => {
                            let name = dict_listing.to_string();
                            let node = Node {
                                name: name.as_str().into(),
                                type_: NodeType::Dict(Default::default()),
                                parent: Some(state.current_directory),
                            };
                            content.insert(name.as_str().into(), node);
                        }
                    }
                }
                _ => {
                    return Err(Error::msg("I am in file?"));
                }
            }
        }
    }
    Ok(())
}

fn parse_line(line: &str) -> Result<ParsedLine, Error> {
    let stripped_command = line.strip_prefix("$ ");
    match stripped_command {
        Some(command) => {
            match command {
                _ if command.starts_with("ls") => {
                    Ok(ParsedLine::Command(Command::ListDirectory))
                }
                _ if command.starts_with("cd") => {
                    let (cmd, target) = command.split_once(" ").context("invalid command")?;
                    if cmd != "cd" {
                        return Err(Error::msg("invalid command"));
                    }
                    match target {
                        "/" => {
                            Ok(ParsedLine::Command(Command::ChangeDirectory(ChangeDirectoryTarget::Root)))
                        }
                        ".." => {
                            Ok(ParsedLine::Command(Command::ChangeDirectory(ChangeDirectoryTarget::Parent)))
                        }
                        name => {
                            Ok(ParsedLine::Command(Command::ChangeDirectory(ChangeDirectoryTarget::Child(name))))
                        }
                    }
                }
                _ => {
                    Err(Error::msg("invalid command"))
                }
            }
        }
        None => {
            let (size_or_dir, name) = line.split_once(" ").context("invalid line")?;
            match size_or_dir {
                "dir" => {
                    Ok(ParsedLine::Listing(ParsedLineListing::DictListing(name)))
                }
                size => {
                    let size = size.parse::<u64>()?;
                    Ok(ParsedLine::Listing(ParsedLineListing::FileListing(FileListing {
                        name,
                        size,
                    })))
                }
            }
        }
    }
}

struct NodeIter<'a, F>
    where
        F: Fn(&Node) -> bool
{
    root: Option<&'a Node>,
    iterator_stack: Vec<Values<'a, String, Node>>,
    predicate: F,
}

impl<'a, F> Iterator for NodeIter<'a, F>
    where
        F: Fn(&Node) -> bool
{
    type Item = &'a Node;

    fn next(&mut self) -> Option<Self::Item> {
        match self.root {
            Some(root) => {
                self.root = None;
                let node = match &root.type_ {
                    NodeType::File(_) => {
                        root
                    }
                    NodeType::Dict(dict) => {
                        self.iterator_stack.push(dict.content.values());
                        root
                    }
                };
                if (self.predicate)(node) {
                    return Some(node);
                }
            }
            None => {}
        }

        let ref mut stack = self.iterator_stack;
        loop {
            match stack.last_mut() {
                None => {
                    return None;
                }
                Some(last) => {
                    let next = last.next();
                    match next {
                        None => {
                            stack.pop();
                            continue;
                        }
                        Some(node) => {
                            let node = match &node.type_ {
                                NodeType::File(_) => {
                                    node
                                }
                                NodeType::Dict(dict) => {
                                    stack.push(dict.content.values());
                                    node
                                }
                            };
                            if (self.predicate)(node) {
                                return Some(node);
                            } else {
                                continue;
                            }
                        }
                    }
                }
            }
        }
    }
}
