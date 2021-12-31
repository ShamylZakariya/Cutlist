use rand::{prelude::SliceRandom, thread_rng};

use super::model;

#[derive(Clone, Debug)]
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

/// Represents a stack of cuts which can be easily crosscut from a board, and then ripped and crosscut to dimension.
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

    pub fn length(&self) -> f32 {
        let mut max_length = 0f32;
        for s in &self.cuts {
            max_length = max_length.max(s.length)
        }
        max_length
    }

    pub fn width(&self) -> f32 {
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

    // returns a score representing how well used the stack is,
    // where 1 means perfect allocaiton without any waste.
    fn score(&self) -> f32 {
        if !self.cuts.is_empty() {
            self.used_area() / self.required_area()
        } else {
            0f32
        }
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
    fn can_accept(&self, cut: &Cut) -> bool {
        self.length >= cut.length
            && self.width >= cut.width
            && self.best_stack_for_cut(cut).is_some()
            && self.unallocated_length() >= cut.length
    }

    // if the board can take this cut into its allocation, take it in, returning true, otherwise return false
    fn accept(&mut self, cut: &Cut) -> bool {
        if cut.length > self.length || cut.width > self.width {
            // cut simply will not fit this board
            false
        } else if let Some(best_stack_index) = self.best_stack_for_cut(cut) {
            // if we found a viable stack for this cut att it
            self.stacks[best_stack_index].add(cut.clone());
            true
        } else if self.unallocated_length() >= cut.length {
            // Create a new stack for this cut
            let mut new_stack = CutStack::new();
            new_stack.add(cut.clone());
            self.stacks.push(new_stack);
            true
        } else {
            false
        }
    }

    // returns the total length unused by stacks
    fn unallocated_length(&self) -> f32 {
        self.length
            - self
                .stacks
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

    fn score(&self) -> Option<f32> {
        if !self.stacks.is_empty() {
            Some(
                self.stacks
                    .iter()
                    .fold(1f32, |acc, stack| acc * stack.score()),
            )
        } else {
            None
        }
    }
}

fn score(boards: &[Board]) -> f32 {
    boards
        .iter()
        .map(|board| board.score())
        .flatten()
        .fold(1f32, |acc, score| acc * score)
}

fn is_a_solution_possible(model: &model::Input) -> bool {
    // if any cut in the cutlist is wider than available board stock,
    // no solution is possible!

    for cut in &model.cutlist {
        for board in &model.boards {
            if cut.width > board.width {
                return false;
            }
        }
    }

    true
}

struct CutRanges {
    longest: f32,
    shortest: f32,
    widest: f32,
    narrowest: f32,
}

/// Returns the index of the best board in `boards` to attempt to insert the cut, or None
fn best_board_for_cut(boards: &[Board], cut: &Cut, cut_ranges: &CutRanges) -> Option<usize> {
    // naive approach - find first board that could accept this cut
    // TODO: Maybe try to put narrow cuts in narrow boards...
    for (i, board) in boards.iter().enumerate() {
        if board.can_accept(cut) {
            return Some(i);
        }
    }

    None
}

/// Vends a new board from the model's board options best suited for the specified cut
fn vend_new_board_for_cut(
    model: &model::Input,
    cut: &Cut,
    cut_ranges: &CutRanges,
) -> Option<Board> {
    // find first board wide enough for this cut
    let mut board_models = model.boards.to_vec();
    board_models.sort_by(|a, b| a.width.partial_cmp(&b.width).unwrap());

    for board_model in &board_models {
        if board_model.width > cut.width && board_model.length > cut.length {
            return Some(board_model.into());
        }
    }

    None
}

fn generate(model: &model::Input, cutlist: &[Cut], cut_ranges: &CutRanges) -> Option<Vec<Board>> {
    let mut cutlist = cutlist.to_vec();

    let mut boards: Vec<Board> = Vec::new();

    while let Some(cut) = cutlist.pop() {
        // Check if there's a decent candidate board
        if let Some(board_index) = best_board_for_cut(&boards, &cut, cut_ranges) {
            if boards[board_index].accept(&cut) {
                continue;
            }
        }

        // See if any of the boards will accept this cut
        for (i, board) in boards.iter_mut().enumerate() {
            if board.accept(&cut) {
                continue;
            }
        }

        // Looks like we need to vend a new board
        if let Some(mut new_board) = vend_new_board_for_cut(model, &cut, cut_ranges) {
            if new_board.accept(&cut) {
                boards.push(new_board);
            } else {
                // This really should not happen as the `is_solution_possible` function should
                // prevent this function from ever running if the model is insufficient to compute a solution.
                return None;
            }
        } else {
            // This also should not occur for same reason as above - `is_solution_possible` should
            // guard against this occurance.
            return None;
        }
    }

    Some(boards)
}

/// Atempts to find a best solution for computing the cutlist for the given model.
pub fn compute(
    model: &model::Input,
    attempts: usize,
    result_count: usize,
) -> Option<Vec<Vec<Board>>> {
    if !is_a_solution_possible(model) {
        return None;
    }

    // Create a vector of our required Cuts, sorted from longest to shortest
    let (mut cutlist, cut_ranges) = {
        let mut cutlist: Vec<Cut> = Vec::new();
        let mut longest: f32 = 0f32;
        let mut widest: f32 = 0f32;
        let mut shortest: f32 = f32::MAX;
        let mut narrowest: f32 = f32::MAX;
        for cut_model in &model.cutlist {
            for _ in 0..cut_model.count {
                longest = longest.max(cut_model.length);
                widest = widest.max(cut_model.width);
                shortest = shortest.min(cut_model.length);
                narrowest = narrowest.min(cut_model.width);
                cutlist.push(cut_model.into());
            }
        }

        (
            cutlist,
            CutRanges {
                longest,
                shortest,
                widest,
                narrowest,
            },
        )
    };

    let mut results = Vec::new();
    let mut rng = thread_rng();

    for attempt in 0..attempts {
        cutlist.shuffle(&mut rng);
        if let Some(result) = generate(model, &cutlist, &cut_ranges) {
            results.push(result);
        }
    }

    if !results.is_empty() {
        // sort results by score with best at front, and then return the desired count
        results.sort_by(|a, b| score(b).partial_cmp(&score(a)).unwrap());
        let result_count = result_count.min(results.len());
        println!("Found {} viable solutions", result_count);
        Some(results[0..result_count].to_vec())
    } else {
        None
    }
}
