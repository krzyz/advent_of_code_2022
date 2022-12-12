#![feature(try_blocks)]

use std::{rc::Rc, str::FromStr};

use anyhow::{anyhow, Context, Result};
use itertools::{iproduct, Itertools};
use petgraph::algo::dijkstra;
use petgraph::prelude::UnGraph;

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
                            Ok('a' as i32 - 1)
                        }
                        'E' => {
                            end = Some((row, col));
                            Ok('z' as i32 + 1)
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
    graph: UnGraph<i32, ()>,
    start: usize,
    end: usize,
}

impl From<Heightmap> for HeightmapGraph {
    fn from(map: Heightmap) -> Self {
        let map = Rc::new(map);
        let edges = iproduct!(0..map.rows(), 0..map.cols())
            .flat_map(|(i, j)| {
                let map = map.clone();
                iproduct!([-1, 1].into_iter(), [-1, 1].into_iter()).flat_map(move |(dx, dy)| {
                    let i2 = i as i32 + dx;
                    let j2 = j as i32 + dy;
                    if let Some((i2, j2)) = (i2 >= 0 && i2 < map.rows() as i32)
                        .then(|| i2)
                        .and_then(|i2| {
                            (j2 >= 0 && j2 < map.cols() as i32).then(|| (i2 as usize, j2 as usize))
                        })
                    {
                        if (map.heights[i][j] - map.heights[i2][j2]).abs() <= 1 {
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
            graph: UnGraph::from_edges(edges.as_slice()),
            start: (map.ind(map.start.0, map.start.1)),
            end: (map.ind(map.end.0, map.end.1)),
        }
    }
}

pub fn get_shortest_path_len(input: &str) -> Result<usize> {
    let heightmap = Heightmap::from_str(input)?;

    let heightmap_graph = HeightmapGraph::from(heightmap);

    let node_map = dijkstra(
        &heightmap_graph.graph,
        (heightmap_graph.start as u32).into(),
        Some((heightmap_graph.end as u32).into()),
        |_| 1,
    );

    println!("{node_map:#?}");

    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_INPUT: &str = include_str!("../data/test_input");

    #[test]
    fn part1() {
        let res = get_shortest_path_len(TEST_INPUT);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 31);
    }
}
