use std::iter;

use itertools::Itertools;

#[derive(Debug)]
pub enum Command {
    Cd(String),
    Ls(Vec<Node>),
}

impl Command {
    pub fn parse_multiple(input: impl Iterator<Item = String>) -> Vec<Self> {
        let mut cur_command: Option<(usize, String)> = None;
        input
            .map(|l| {
                let l: String = l.into();
                l
            })
            .group_by(|l| {
                if l.starts_with('$') {
                    let command_nth = if let Some((i, _)) = cur_command { i } else { 0 };

                    cur_command = Some((
                        command_nth + 1,
                        l.chars()
                            .skip(1)
                            .skip_while(|c| c.is_whitespace())
                            .collect::<String>(),
                    ));
                }
                cur_command
                    .clone()
                    .expect("Missing command at the beginning of input!")
            })
            .into_iter()
            .map(|(command_str, output)| Self::parse(command_str.1, output.skip(1)))
            .collect::<Vec<Self>>()
    }

    fn parse(command_str: String, command_output: impl Iterator<Item = String>) -> Self {
        let mut command_split = command_str.split(char::is_whitespace);
        match command_split.next() {
            Some("cd") => Command::Cd(
                command_split
                    .next()
                    .expect("Missing cd argument")
                    .to_string(),
            ),
            Some("ls") => Command::Ls(
                command_output
                    .map(|l| {
                        let mut lsplit = l.split(char::is_whitespace);
                        match lsplit.next() {
                            Some("dir") => {
                                Node::new_dir(lsplit.next().expect("Missing dir name").to_string())
                            }
                            Some(s) => {
                                let size = s.parse().expect("Size of a file in a wrong format!");
                                let name = lsplit.next().expect("Missing file name").to_string();
                                Node::new_file(name, size)
                            }
                            _ => panic!("Unrecognized ls output!"),
                        }
                    })
                    .collect(),
            ),
            _ => panic!("Unrecognized command: {command_str}"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Node {
    File { name: String, size: usize },
    Dir { name: String, contents: Vec<Node> },
}

impl Node {
    fn new_dir(name: String) -> Self {
        Self::Dir {
            name,
            contents: vec![],
        }
    }

    fn new_file(name: String, size: usize) -> Self {
        Self::File { size, name }
    }

    fn size(&self) -> usize {
        match self {
            Node::File { size, .. } => *size,
            Node::Dir { contents, .. } => contents.iter().map(|n| n.size()).sum(),
        }
    }

    fn get_sizes(&self) -> Vec<(String, usize)> {
        if let Node::Dir { name, contents } = self {
            contents
                .iter()
                .map(|n| Node::get_sizes(n))
                .flatten()
                .chain(iter::once((name.clone(), self.size())))
                .collect::<Vec<_>>()
        } else {
            vec![]
        }
    }

    fn get_node_by_location_mut(&mut self, location: &[String]) -> &mut Node {
        let mut current_node = self;
        for dir_name in location.iter().skip_while(|l| l.as_str() == "/") {
            let next_node = if let Node::Dir { contents, .. } = current_node {
                contents
                    .iter_mut()
                    .find(|n| {
                        if let Node::Dir { name, .. } = n {
                            name.as_str() == dir_name
                        } else {
                            false
                        }
                    })
                    .expect(format!("Unable to cd into {dir_name}").as_str())
            } else {
                panic!("next node is not a directory!");
            };

            current_node = next_node;
        }

        current_node
    }

    fn parse(input: impl Iterator<Item = String>) -> Self {
        let commands = Command::parse_multiple(input);

        let root_str = "/".to_string();

        let mut root = Node::new_dir(root_str.clone());
        let mut current_location: Vec<String> = vec![];

        for command in commands {
            (root, current_location) = match command {
                Command::Cd(cd_to) => match cd_to.as_str() {
                    ".." => {
                        let mut new_location = current_location.clone();
                        new_location
                            .pop()
                            .expect("Unable to go up from root directory!");
                        (root.clone(), new_location)
                    }
                    dir_name => {
                        let mut new_location = current_location.clone();
                        if let Some(_) = current_location.last() {
                            new_location.push(dir_name.to_string());
                        } else {
                            if dir_name != root_str.as_str() {
                                panic!("First cd is not into root!")
                            }
                            new_location.push(root_str.clone());
                        }
                        (root.clone(), new_location)
                    }
                },
                Command::Ls(nodes) => {
                    let mut new_root = root.clone();
                    if let Node::Dir { contents, .. } =
                        new_root.get_node_by_location_mut(current_location.as_slice())
                    {
                        *contents = nodes
                    };
                    (new_root, current_location.clone())
                }
            };
        }

        root
    }
}

pub fn size_smallest(input: impl Iterator<Item = String>, biggest: usize) -> usize {
    let root = Node::parse(input);
    let dirs_with_sizes = root.get_sizes();

    dirs_with_sizes
        .iter()
        .filter_map(|(_, size)| if size < &biggest { Some(size) } else { None })
        .sum()
}

pub fn size_to_delete(input: impl Iterator<Item = String>, total: usize, needed: usize) -> usize {
    let root = Node::parse(input);
    let sizes = root
        .get_sizes()
        .into_iter()
        .map(|(_, size)| size)
        .collect::<Vec<_>>();

    let used = sizes
        .iter()
        .copied()
        .max()
        .expect("No directories sizes found!");

    let to_free = needed
        .checked_sub(total.checked_sub(used).expect("Used more than total!"))
        .expect("More than enough space!");

    sizes
        .iter()
        .copied()
        .filter(|&s| s >= to_free)
        .min()
        .expect("No directory to free found")
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_INPUT: &str = include_str!("../data/test_input");

    #[test]
    fn part1() {
        let res = size_smallest(TEST_INPUT.lines().map(|l| l.to_string()), 100000);
        assert_eq!(res, 95437);
    }

    #[test]
    fn part2() {
        let res = size_to_delete(
            TEST_INPUT.lines().map(|l| l.to_string()),
            70000000,
            30000000,
        );
        assert_eq!(res, 24933642);
    }
}
