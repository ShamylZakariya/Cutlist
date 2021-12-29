use itertools::Itertools;
use rand::{thread_rng, prelude::SliceRandom};

use super::model;



#[derive(Clone)]
pub struct Cut {
    pub length: f32,
    pub width: f32,
    pub id: String,
}

impl PartialEq for Cut {
    fn eq(&self, other: &Self) -> bool {
        self.length == other.length && self.width == other.width && self.id == other.id
    }
}

impl Eq for Cut {}

impl std::hash::Hash for Cut {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let length = (self.length * 512f32).floor() as i64;
        let width = (self.width * 512f32).floor() as i64;

        length.hash(state);
        width.hash(state);
        self.id.hash(state);
    }
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

fn score(boards: &[Board]) -> f32 {
    boards.iter().fold(1f32, |acc, board| acc * board.score())
}

fn is_a_solution_possible(boards: &[Board], cutlist: &[Cut]) -> bool {
    let total_board_area = boards.iter().map(|board| board.length * board.width).fold(0f32, |acc, area| acc + area);
    let total_cutlist_area = cutlist.iter().map(|cut| cut.width * cut.length).fold(0f32, |acc, area| acc + area);
    total_cutlist_area <= total_board_area
}

fn perform_cutlist_allocation(boards: &[Board], cutlist: &[&Cut], spacing: f32) -> Option<Vec<Board>> {
    let mut boards = boards.to_vec();
    let mut cuts = cutlist.to_vec();
    let mut orphaned_cuts = Vec::new();

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

fn factorial(x:u32) -> u128 {
    let mut f = 1u128;
    for i in 1..=x as u128 {
        f = f * i;
    }
    f
}

/// Atempts to find a best solution for computing the cutlist for the given model.
pub fn compute(model: &model::Input, attempts: usize) -> Option<Vec<Board>> {
    let mut results = vec![];
    let boards: Vec<Board> = model.boards.iter().map(|board| board.into()).collect();

    let mut cutlist: Vec<Cut> = Vec::new();
    for cut_model in &model.cutlist {
        for _ in 0 .. cut_model.count {
            cutlist.push(cut_model.into());
        }
    }

    if !is_a_solution_possible(&boards, &cutlist) {
        println!("No solution is possible");
        return None
    }

    if attempts == 0 {
        println!("We have {} cuts, which will be {} permutations...", cutlist.len(), factorial(cutlist.len() as u32));

        let mut attempts: usize = 0;
        for cutlist in cutlist.iter().permutations(cutlist.len()).unique() {
            attempts += 1;
            if let Some(result) = perform_cutlist_allocation(&boards, &cutlist, model.spacing) {
                println!("Found a result");
                results.push(result);
            }
        }
    } else {
        let mut rng = thread_rng();
        for _ in 0..attempts {
            cutlist.shuffle(&mut rng);
            let cutlist_ref: Vec<&Cut> = cutlist.iter().collect();
            if let Some(result) = perform_cutlist_allocation(&boards, &cutlist_ref, model.spacing) {
                println!("Found a result");
                results.push(result);
            }
        }
    }

    let scores: Vec<_> = results
        .iter()
        .map(|solution| score(solution))
        .map(|score| score.to_string())
        .collect();

    println!(
        "Out of {} attempts, found: {} viable solutions, with scores: [{}]",
        attempts,
        results.len(),
        scores.join(", ")
    );

    // sort results by score and pick the best
    results.sort_by(|a, b| score(a).partial_cmp(&score(b)).unwrap());

    if let Some(result) = results.last() {
        println!("Best result had score: {}", score(result));
        Some(result.clone())
    } else {
        None
    }
}
