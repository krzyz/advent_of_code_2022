#![feature(int_roundings)]

use std::{
    array::IntoIter,
    collections::HashMap,
    fmt,
    iter::{Cycle, Enumerate, Peekable},
};

use anyhow::{anyhow, Result};
use static_init::{dynamic, LazyAccess};

#[dynamic]
static ROCK1: Rock = Rock {
    occupied: vec![vec![true, true, true, true]],
};

#[dynamic]
static ROCK2: Rock = Rock {
    occupied: vec![
        vec![false, true, false],
        vec![true, true, true],
        vec![false, true, false],
    ],
};

#[dynamic]
static ROCK3: Rock = Rock {
    occupied: vec![
        vec![true, true, true],
        vec![false, false, true],
        vec![false, false, true],
    ],
};

#[dynamic]
static ROCK4: Rock = Rock {
    occupied: vec![vec![true], vec![true], vec![true], vec![true]],
};

#[dynamic]
static ROCK5: Rock = Rock {
    occupied: vec![vec![true, true], vec![true, true]],
};

#[derive(Debug, PartialEq, Eq)]
struct Rock {
    // filled, x -> right, y -> top
    // starting with bottom left corner
    occupied: Vec<Vec<bool>>,
}

impl Rock {
    fn falling_rocks_iter_cycle() -> Cycle<Enumerate<IntoIter<&'static Rock, 5>>> {
        let x = [
            LazyAccess::get(&ROCK1),
            LazyAccess::get(&ROCK2),
            LazyAccess::get(&ROCK3),
            LazyAccess::get(&ROCK4),
            LazyAccess::get(&ROCK5),
        ]
        .into_iter()
        .enumerate()
        .cycle();
        x
    }

    fn len(&self) -> usize {
        self.occupied.iter().map(|row| row.len()).max().unwrap()
    }
}

struct Chamber {
    occupied: Vec<Vec<bool>>,
    unseen_height: i64,
}

impl fmt::Display for Chamber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in self.occupied.iter().rev() {
            for &x in row.iter() {
                if x {
                    write!(f, "██")?;
                } else {
                    write!(f, "░░")?;
                }
            }
            writeln!(f, "")?;
        }

        Ok(())
    }
}

impl Chamber {
    fn is_colliding(&self, rock_state: &RockState) -> bool {
        if (rock_state.pos.1 + self.occupied.len() as i32) < 0 {
            return true;
        }
        rock_state
            .rock
            .occupied
            .iter()
            .enumerate()
            .map(|(i, row)| (i as i32 + rock_state.pos.1, row))
            .filter(|(y, _)| y < &0)
            .flat_map(|(y, row)| {
                row.iter().enumerate().filter_map(move |(j, occupied)| {
                    occupied.then(|| {
                        let x = j as i32 + rock_state.pos.0;
                        let i_index: usize = (self.occupied.len() as i32 + y).try_into().ok()?;
                        let j_index: usize = x.try_into().ok()?;
                        let r = self.occupied.get(i_index)?.get(j_index)?;
                        r.then_some(())
                    })?
                })
            })
            .next()
            .is_some()
    }
}

#[derive(Debug, Clone, Copy)]
enum JetDirection {
    Left,
    Right,
}

impl TryFrom<char> for JetDirection {
    type Error = anyhow::Error;

    fn try_from(s: char) -> Result<Self> {
        match s {
            '<' => Ok(JetDirection::Left),
            '>' => Ok(JetDirection::Right),
            _ => Err(anyhow!(format!("Unrecognized jet direction: {s}"))),
        }
    }
}

struct CycleDescription {
    rocks_in_between: i64,
    height_in_between: i64,
}

struct FallSimulation {
    chamber: Chamber,
    rocks_fallen: i64,
    max_rocks: i64,
    rock_iterator: Peekable<Cycle<Enumerate<IntoIter<&'static Rock, 5>>>>,
    jet_iterator: Peekable<Cycle<Enumerate<std::vec::IntoIter<JetDirection>>>>,
    tops: HashMap<(usize, usize, Vec<Vec<bool>>), (i64, i64)>,
}

impl FallSimulation {
    fn new(max_rocks: i64, jet_directions: Vec<JetDirection>) -> FallSimulation {
        let chamber = Chamber {
            occupied: vec![],
            unseen_height: 0,
        };
        let rocks_fallen = 0;
        let rock_iterator = Rock::falling_rocks_iter_cycle().peekable();
        let jet_iterator = jet_directions.into_iter().enumerate().cycle().peekable();

        FallSimulation {
            chamber,
            rocks_fallen,
            max_rocks,
            rock_iterator,
            jet_iterator,
            tops: HashMap::new(),
        }
    }

    // returns some if it's a cycle
    fn add_fallen(&mut self, rock_state: RockState) -> Option<CycleDescription> {
        self.rocks_fallen += 1;

        let mut modified = vec![];

        for (i, row) in rock_state.rock.occupied.iter().enumerate() {
            let y = i as i32 + rock_state.pos.1;

            if y < 0 {
                for (j, occupied) in row.iter().enumerate().filter(|(_, occupied)| **occupied) {
                    let x = j as i32 + rock_state.pos.0;
                    let i_index: usize =
                        (self.chamber.occupied.len() as i32 + y).try_into().unwrap();
                    let j_index: usize = x.try_into().unwrap();

                    modified.push(j_index);

                    *self
                        .chamber
                        .occupied
                        .get_mut(i_index)
                        .unwrap()
                        .get_mut(j_index)
                        .unwrap() = *occupied;
                }
            } else {
                let new_row = (0..7)
                    .map(|i| i as i32 - rock_state.pos.0)
                    .map(|i| {
                        (i >= 0)
                            .then_some(())
                            .and(row.get(i as usize).copied())
                            .unwrap_or(false)
                    })
                    .collect::<Vec<_>>();

                self.chamber.occupied.push(new_row);
            }
        }

        let i_jet = self.jet_iterator.peek().unwrap().0;
        let i_rock = self.rock_iterator.peek().unwrap().0;

        let last_four = self
            .chamber
            .occupied
            .iter()
            .rev()
            .take(8)
            .rev()
            .cloned()
            .collect::<Vec<_>>();

        let height_now = self.chamber.unseen_height + self.chamber.occupied.len() as i64;

        if let Some((old_rocks, old_height)) = self
            .tops
            .insert((i_jet, i_rock, last_four), (self.rocks_fallen, height_now))
        {
            Some(CycleDescription {
                rocks_in_between: self.rocks_fallen - old_rocks,
                height_in_between: height_now - old_height,
            })
        } else {
            None
        }
    }

    fn progress(&mut self, state: State) -> State {
        match state {
            State::Blowing(state) => match self.jet_iterator.next() {
                Some((_, direction)) => {
                    let new_state = match direction {
                        JetDirection::Left => state.moved((-1, 0)).unwrap_or(state),
                        JetDirection::Right => state.moved((1, 0)).unwrap_or(state),
                    };
                    if self.chamber.is_colliding(&new_state) {
                        State::Falling(state)
                    } else {
                        State::Falling(new_state)
                    }
                }
                None => State::Falling(state),
            },
            State::Falling(state) => {
                let new_state = state.moved((0, -1)).unwrap();
                if self.chamber.is_colliding(&new_state) {
                    if let Some(CycleDescription {
                        rocks_in_between,
                        height_in_between,
                    }) = self.add_fallen(state)
                    {
                        let i_cycles =
                            (self.max_rocks - self.rocks_fallen).div_floor(rocks_in_between);
                        self.rocks_fallen += i_cycles * rocks_in_between;
                        self.chamber.unseen_height += i_cycles * height_in_between;
                    }
                    State::NewRock
                } else {
                    State::Blowing(new_state)
                }
            }
            State::NewRock => {
                if self.rocks_fallen == self.max_rocks {
                    State::End
                } else {
                    State::Blowing(RockState {
                        pos: (2, 3),
                        rock: self.rock_iterator.next().unwrap().1,
                    })
                }
            }
            State::End => State::End,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RockState {
    pos: (i32, i32),
    rock: &'static Rock,
}

impl RockState {
    fn moved(&self, delta: (i32, i32)) -> Option<Self> {
        let RockState { pos, rock } = self;
        let new_x = pos.0 + delta.0;
        (new_x >= 0 && new_x + (self.rock.len() as i32) <= 7).then(|| ())?;
        let new_y = pos.1 + delta.1;
        Some(RockState {
            pos: (new_x, new_y),
            rock,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum State {
    Blowing(RockState),
    Falling(RockState),
    NewRock,
    End,
}

pub fn get_tower_height(mut input: impl Iterator<Item = String>, num_rocks: i64) -> Result<i64> {
    let jet_directions = input
        .next()
        .ok_or(anyhow!["Missing jet directions input!"])?
        .chars()
        .map(TryFrom::try_from)
        .collect::<Result<Vec<JetDirection>>>()?;

    let mut fall_simulation = FallSimulation::new(num_rocks, jet_directions);

    let mut state = State::NewRock;
    while state != State::End {
        state = fall_simulation.progress(state);
    }

    Ok(fall_simulation.chamber.occupied.len() as i64 + fall_simulation.chamber.unseen_height)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_INPUT: &str = include_str!("../data/test_input");

    #[test]
    fn part1() {
        let res = get_tower_height(TEST_INPUT.lines().map(|l| l.to_string()), 2022);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 3068);
    }

    #[test]
    fn part2() {
        let res = get_tower_height(TEST_INPUT.lines().map(|l| l.to_string()), 1000000000000);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1514285714288);
    }
}
