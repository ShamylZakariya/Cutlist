use anyhow::{bail, Result};
use macroquad::prelude::*;
use ::rand::seq::SliceRandom;
use ::rand::thread_rng;

use crate::model;

#[derive(Clone)]
pub struct Cut {
    pub length: f32,
    pub width: f32,
    pub id: String,
}

impl From<&model::Cut> for Cut {
    fn from(cut: &model::Cut) -> Self {
        Cut {
            length: cut.length,
            width: cut.width,
            id: cut.name.clone(),
        }
    }
}

impl Cut {
    fn rotate(self) -> Cut {
        Cut {
            length: self.width,
            width: self.length,
            id: self.id,
        }
    }
}

#[derive(Clone)]
pub struct CutStack {
    pub cuts: Vec<Cut>,
}

impl CutStack {
    fn new() -> Self {
        Self { cuts: Vec::new() }
    }

    fn add(&mut self, cut: Cut) {
        self.cuts.push(cut);
    }

    fn length(&self) -> f32 {
        let mut max_length = 0f32;
        for s in &self.cuts {
            max_length = max_length.max(s.length)
        }
        max_length
    }

    fn width(&self) -> f32 {
        self.cuts.iter().map(|s| s.width).sum()
    }

    // returns area required to represent the cuts in this stack
    fn required_area(&self) -> f32 {
        self.length() * self.width()
    }

    // returns actual area used by the cuts in this stack
    fn used_area(&self) -> f32 {
        let mut area = 0f32;
        for c in &self.cuts {
            area += c.width * c.length
        }
        area
    }
}

#[derive(Clone)]
pub struct Board {
    pub length: f32,
    pub width: f32,
    pub id: String,
    pub stacks: Vec<CutStack>,
}

impl From<&model::Board> for Board {
    fn from(board: &model::Board) -> Self {
        Board {
            length: board.length,
            width: board.width,
            id: board.id.clone(),
            stacks: Vec::new(),
        }
    }
}

impl Board {
    // if the board can take this cut into its allocation, take it in, returning true, otherwise return false
    fn accept(&mut self, cut: &Cut) -> bool {
        if cut.length > self.length || cut.width > self.width {
            // cut simply will not fit this board
            false
        } else if let Some(best_stack_index) = self.best_stack_for_cut(cut) {
            // if we found a viable stack for this cut att it
            self.stacks[best_stack_index].add(cut.clone());
            true
        } else if self.total_allocated_length() <= cut.length {
            // see if we can create a new stack
            let mut new_stack = CutStack::new();
            new_stack.add(cut.clone());
            self.stacks.push(new_stack);
            true
        } else {
            false
        }
    }

    // returns the total length of the board consumed by its stacks
    fn total_allocated_length(&self) -> f32 {
        self.stacks
            .iter()
            .fold(0f32, |acc, stack| acc + stack.length())
    }

    fn best_stack_for_cut(&self, cut: &Cut) -> Option<usize> {
        // find the best stack in the board for this cut
        // TODO: Consider a vetting criteria such as, is this stack less than 50% different in length?
        let mut best_stack_index: Option<usize> = None;
        let mut best_stack_length_difference: f32 = f32::MAX;
        for (i, stack) in self.stacks.iter().enumerate() {
            if stack.width() + cut.width < self.width {
                let length_difference = (cut.length - stack.length()).abs();
                if length_difference < best_stack_length_difference {
                    best_stack_index = Some(i);
                    best_stack_length_difference = length_difference;
                }
            }
        }

        best_stack_index
    }

    /// Returns a scoring value from 0 to 1 representing how well utilized the board is; the metric is
    /// based on how densely packed the cuts are, and how much usable scrap would be left over.
    fn score(&self) -> f32 {
        let board_area = self.length * self.width;
        let required_area: f32 = self.stacks.iter().map(|s| s.required_area()).sum();
        let used_area: f32 = self.stacks.iter().map(|s| s.used_area()).sum();
        let scrap = board_area - required_area;

        if required_area > 0f32 {
            let density = (used_area / required_area).clamp(0f32, 1f32);
            let scrap_density = scrap / board_area;
            density * scrap_density // total score
        } else {
            // an unused board is pure scrap, so it's worth a lot
            // TODO: Is this the right way to score?
            1f32
        }
    }
}

fn score(cutlist: &[Board]) -> f32 {
    let mut total_score: f32 = 1f32;
    for board in cutlist {
        total_score *= board.score();
    }
    total_score
}

pub fn compute(model: &model::Input, attempts: usize) -> Result<Vec<Board>> {
    let mut results = vec![];
    for attempt in 0..attempts {
        if let Some(result) = perform_cutlist_allocation(model, attempt) {
            results.push(result);
        }
    }

    let scores:Vec<_> = results.iter()
        .map(|solution| score(solution))
        .map(|score| score.to_string())
        .collect();

    println!("Out of {} attempts, found: {} viable solutions, with scores: [{}]", attempts, results.len(), scores.join(", "));

    // sort results by a scoring metric and pick the best
    results.sort_by(|a, b| score(a).partial_cmp(&score(b)).unwrap());

    if let Some(result) = results.first() {
        Ok(result.clone())
    } else {
        bail!("Couldn't find a viable solution")
    }
}

fn perform_cutlist_allocation(model: &model::Input, attempt: usize) -> Option<Vec<Board>> {
    let mut rng = thread_rng();
    let mut cuts: Vec<Cut> = model.cutlist.iter().map(|c| c.into()).collect();
    cuts.shuffle(&mut rng);

    let mut boards: Vec<Board> = model.boards.iter().map(|b| b.into()).collect();
    let mut orphaned_cuts: Vec<Cut> = Vec::new();

    while let Some(cut) = cuts.pop() {
        let mut orphaned = true;
        for board in &mut boards {
            if board.accept(&cut) {
                orphaned = false;
                break;
            }
        }
        if orphaned {
            orphaned_cuts.push(cut);
        }
    }

    if orphaned_cuts.is_empty() {
        Some(boards)
    } else {
        None
    }
}
