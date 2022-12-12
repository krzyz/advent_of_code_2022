#![feature(try_blocks)]

use std::{rc::Rc, str::FromStr};

use anyhow::{anyhow, Context, Result};
use itertools::{iproduct, Itertools};
use petgraph::algo::dijkstra;
use petgraph::graph::NodeIndex;
use petgraph::prelude::Graph;

#[derive(Debug, Clone)]
struct Heightmap {
    heights: Vec<Vec<i32>>,
    start: (usize, usize),
    end: (usize, usize),
}

impl Heightmap {
    fn rows(&self) -> usize {
        self.heights.len()
    }

    fn cols(&self) -> usize {
        self.heights[0].len()
    }

    fn ind(&self, row: usize, col: usize) -> usize {
        row * self.cols() + col
    }
}

impl FromStr for Heightmap {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (mut start, mut end) = (None, None);
        let heights = s
            .lines()
            .enumerate()
            .map(|(row, l)| try {
                l.chars()
                    .enumerate()
                    .map(|(col, c)| match c {
                        'S' => {
                            start = Some((row, col));
                            Ok('a' as i32)
                        }
                        'E' => {
                            end = Some((row, col));
                            Ok('z' as i32)
                        }
                        'a'..='z' => Ok(c as i32),
                        _ => Err(anyhow!(format!("Unrecognized height character: {c}"))),
                    })
                    .collect::<Result<Vec<i32>>>()?
            })
            .collect::<Result<Vec<_>>>()?;

        heights
            .iter()
            .map(|row| row.len())
            .all_equal()
            .then(|| ())
            .context("Map row lengths are not equal!")?;

        Ok(Self {
            heights,
            start: start.ok_or(anyhow!("Missing start point!"))?,
            end: end.ok_or(anyhow!("Missing end point!"))?,
        })
    }
}

#[derive(Debug, Clone)]
struct HeightmapGraph {
    graph: Graph<i32, ()>,
    end: usize,
}

impl From<Heightmap> for HeightmapGraph {
    fn from(map: Heightmap) -> Self {
        let map = Rc::new(map);
        let edges = iproduct!(0..map.rows(), 0..map.cols())
            .flat_map(|(i, j)| {
                let map = map.clone();
                [(-1, 0), (1, 0), (0, -1), (0, 1)]
                    .iter()
                    .flat_map(move |(dx, dy)| {
                        let i2 = i as i32 + dx;
                        let j2 = j as i32 + dy;
                        if let Some((i2, j2)) = (i2 >= 0 && i2 < map.rows() as i32)
                            .then(|| i2)
                            .and_then(|i2| {
                                (j2 >= 0 && j2 < map.cols() as i32)
                                    .then(|| (i2 as usize, j2 as usize))
                            })
                        {
                            if map.heights[i2][j2] - map.heights[i][j] <= 1 {
                                Some((map.ind(i, j) as u32, map.ind(i2, j2) as u32))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
            })
            .collect::<Vec<_>>();

        Self {
            graph: Graph::from_edges(edges.as_slice()),
            end: (map.ind(map.end.0, map.end.1)),
        }
    }
}

pub fn get_shortest_path_len(input: &str, all: bool) -> Result<i32> {
    let heightmap = Heightmap::from_str(input)?;
    let heightmap = Rc::new(heightmap);

    let starting_nodes = if all {
        heightmap
            .heights
            .iter()
            .enumerate()
            .flat_map(|(i, row)| {
                let heightmap = heightmap.clone();
                row.iter().enumerate().filter_map(move |(j, val)| {
                    if *val == ('a' as i32) {
                        Some((heightmap.ind(i, j) as u32).into())
                    } else {
                        None
                    }
                })
            })
            .collect::<Vec<_>>()
    } else {
        vec![(heightmap.ind(heightmap.start.0, heightmap.start.1) as u32).into()]
    };

    let heightmap_graph = HeightmapGraph::from(Rc::try_unwrap(heightmap).unwrap());

    let mut paths = starting_nodes
        .into_iter()
        .filter_map(|starting_node| {
            let node_map = dijkstra(
                &heightmap_graph.graph,
                starting_node,
                Some((heightmap_graph.end as u32).into()),
                |_| 1,
            );

            node_map
                .get(&NodeIndex::new(heightmap_graph.end as usize))
                .copied()
        })
        .collect::<Vec<_>>();

    paths.sort();

    paths.first().ok_or(anyhow!("No resuls!")).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_INPUT: &str = include_str!("../data/test_input");

    #[test]
    fn part1() {
        let res = get_shortest_path_len(TEST_INPUT, false);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 31);
    }

    #[test]
    fn part2() {
        let res = get_shortest_path_len(TEST_INPUT, true);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 29);
    }
}
