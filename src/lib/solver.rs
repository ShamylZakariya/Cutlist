use rand::prelude::*;
use rand_pcg::Pcg64;

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
        let length = (self.length * 512_f32).floor() as i64;
        let width = (self.width * 512_f32).floor() as i64;

        length.hash(state);
        width.hash(state);
        self.id.hash(state);
    }
}

impl Cut {
    fn from(cut: &model::Cut, outset: f32) -> Cut {
        Cut {
            length: cut.length + outset,
            width: cut.width + outset,
            id: cut.name.clone(),
        }
    }

    fn rotate(self) -> Cut {
        Cut {
            length: self.width,
            width: self.length,
            id: self.id,
        }
    }

    fn area(&self) -> f32 {
        self.width * self.length
    }
}

pub trait Stack {
    fn length(&self) -> f32;

    fn width(&self) -> f32;

    fn area(&self) -> f32 {
        self.length() * self.width()
    }

    fn used_area(&self) -> f32;

    fn is_empty(&self) -> bool;

    fn score(&self) -> f32 {
        if !self.is_empty() {
            let area = self.area();
            if area > 0_f32 {
                self.used_area() / area
            } else {
                0_f32
            }
        } else {
            0_f32
        }
    }

    fn accept(&mut self, cut: Cut);

    fn remove(&mut self, cut: &Cut) -> bool;
}

/// Board is the primary set of crosscuts a board will undergo.
/// Board doesn't contain any Cut instances, rather, each Cut is stored in
/// a RipStack.

/// Board arranges RipStacks from left-to-right. The
/// RipStacks, in turn, stack Cuts from top to bottom
/// -----------------------------------------------------------------------------------------
/// | Board
/// | |Rip Stack | |RipStack |
/// | |  Cut     | | Cut     |
/// | |  Cut     | | CUt     |
/// -----------------------------------------------------------------------------------------
#[derive(Clone, Debug)]
pub struct Board {
    pub length: f32,
    pub width: f32,
    pub id: String,
    pub stacks: Vec<RipStack>,
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
        self.width >= cut.width
            && self.length >= cut.length
            && (self.best_stack_for_cut(cut).is_some() || self.unallocated_length() >= cut.length)
    }

    // if the board can take this cut into its allocation, take it in, returning true, otherwise return false
    fn accept(&mut self, cut: &Cut) -> bool {
        if cut.length > self.length || cut.width > self.width {
            // cut simply will not fit this board
            return false;
        } else if let Some(best_stack_index) = self.best_stack_for_cut(cut) {
            // if we found a viable stack for this cut add it
            // (provided the addition would not overflow board length)
            self.stacks[best_stack_index].accept(cut.clone());
            if self.allocated_length() > self.length {
                self.stacks[best_stack_index].remove(cut);
                return false;
            }

            return true;
        }

        if self.unallocated_length() >= cut.length {
            // No stack can accept the cut; create a new stack
            let mut new_stack = RipStack::new();
            new_stack.accept(cut.clone());
            self.stacks.push(new_stack);
            true
        } else {
            false
        }
    }

    // total length used by stacks
    fn allocated_length(&self) -> f32 {
        self.stacks
            .iter()
            .fold(0_f32, |acc, stack| acc + stack.length())
    }

    // returns the total length unused by stacks
    fn unallocated_length(&self) -> f32 {
        self.length - self.allocated_length()
    }

    // find the best stack in the board for this cut, or None if a new stack should
    // be created
    fn best_stack_for_cut(&self, cut: &Cut) -> Option<usize> {
        // while we have room for a cut, add a new ripstack. When
        // out of room, start adding to ripstacks which are a good fit.
        if self.unallocated_length() >= cut.length {
            None
        } else {
            // select the stack with a length closest to length of current cut
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

            // if the best fitting stack has a length difference more than
            // 50% off our cut length, don't use it
            if best_stack_length_difference < cut.length / 2_f32 {
                best_stack_index
            } else {
                None
            }
        }
    }

    fn score(&self) -> Option<f32> {
        if !self.stacks.is_empty() {
            Some(
                self.stacks
                    .iter()
                    .fold(1_f32, |acc, stack| acc * stack.score()),
            )
        } else {
            None
        }
    }
}

/// RipStack represents a crosscut section from a board which can then be ripped to width.
/// RipStacks are owned by a Board
#[derive(Clone, Debug)]
pub struct RipStack {
    pub stacks: Vec<CrosscutStack>,
}

impl RipStack {
    fn new() -> Self {
        Self { stacks: Vec::new() }
    }

    fn best_stack_for_cut(&self, cut: &Cut) -> Option<usize> {
        None

        // For now, select the shortest stack. TODO: We may want to bias to
        // stacks of closest width to the cut.

        // let mut best_stack_index: Option<usize> = None;
        // let mut best_stack_length: f32 = f32::MAX;

        // for (i, stack) in self.stacks.iter().enumerate() {
        //     let stack_length = stack.length();
        //     if stack_length < best_stack_length {
        //         best_stack_index = Some(i);
        //         best_stack_length = stack_length;
        //     }
        // }

        // best_stack_index
    }
}

impl Stack for RipStack {
    fn accept(&mut self, cut: Cut) {
        // Find the Crosscut Stack which best accepts this cut

        if let Some(best_stack_index) = self.best_stack_for_cut(&cut) {
            self.stacks[best_stack_index].accept(cut.clone());
        } else {
            let mut stack = CrosscutStack::new();
            stack.accept(cut);
            self.stacks.push(stack);
        }
    }

    fn remove(&mut self, cut: &Cut) -> bool {
        for stack in &mut self.stacks {
            if stack.remove(cut) {
                return true;
            }
        }
        false
    }

    // A RipStack is a stack of cuts (top to bottom) which can be ripped from a crosscut section of board.
    // The length of a RipStack is the max length of the contained cuts
    fn length(&self) -> f32 {
        let mut max_length = 0_f32;
        for s in &self.stacks {
            max_length = max_length.max(s.length())
        }
        max_length
    }

    // Given that the RipStack is a stack of cuts, the width is the sum of the cut widths
    fn width(&self) -> f32 {
        self.stacks.iter().map(|s| s.width()).sum()
    }

    fn is_empty(&self) -> bool {
        self.stacks.is_empty()
    }

    fn used_area(&self) -> f32 {
        self.stacks.iter().map(|s| s.used_area()).sum()
    }
}

#[derive(Clone, Debug)]
pub struct CrosscutStack {
    pub stack: Vec<Cut>,
}

impl CrosscutStack {
    fn new() -> Self {
        Self { stack: Vec::new() }
    }
}

impl Stack for CrosscutStack {
    fn accept(&mut self, cut: Cut) {
        self.stack.push(cut);
    }

    fn remove(&mut self, cut: &Cut) -> bool {
        if self.stack.contains(cut) {
            self.stack.retain(|c| c != cut);
            true
        } else {
            false
        }
    }

    // A CrosscutStack a stack of cuts arranged from left to right, which will be crosscut from a rip
    // The length of a RipStack is the sum of cut lengths
    fn length(&self) -> f32 {
        self.stack.iter().map(|s| s.length).sum()
    }

    // The width of CrosscutStack is the max of the widths of cuts
    fn width(&self) -> f32 {
        let mut max_width = 0_f32;
        for s in &self.stack {
            max_width = max_width.max(s.width)
        }
        max_width
    }

    fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    fn used_area(&self) -> f32 {
        let mut area = 0_f32;
        for c in &self.stack {
            area += c.width * c.length
        }
        area
    }
}

///////////////////////////////////////////////////////////////////////////////

pub fn score(boards: &[Board]) -> f32 {
    boards
        .iter()
        .filter_map(|board| board.score())
        .fold(1_f32, |acc, score| acc * score)
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
    // sort boards by increasing width, and find first board wide enough for this cut
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

    'cutlist: while let Some(cut) = cutlist.pop() {
        // Check if there's a decent candidate board
        if let Some(board_index) = best_board_for_cut(&boards, &cut, cut_ranges) {
            if boards[board_index].accept(&cut) {
                continue 'cutlist;
            }
        }

        // See if any of the boards will accept this cut
        for (i, board) in boards.iter_mut().enumerate() {
            if board.accept(&cut) {
                continue 'cutlist;
            }
        }

        // Looks like we need to vend a new board
        if let Some(mut new_board) = vend_new_board_for_cut(model, &cut, cut_ranges) {
            if new_board.accept(&cut) {
                boards.push(new_board);
                continue 'cutlist;
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

    // Create a vector of our required Cuts and collect min/maxes for dimensions
    let (mut cutlist, cut_ranges) = {
        let mut cutlist: Vec<Cut> = Vec::new();
        let mut longest: f32 = 0_f32;
        let mut widest: f32 = 0_f32;
        let mut shortest: f32 = f32::MAX;
        let mut narrowest: f32 = f32::MAX;
        for cut_model in &model.cutlist {
            for _ in 0..cut_model.count {
                longest = longest.max(cut_model.length);
                widest = widest.max(cut_model.width);
                shortest = shortest.min(cut_model.length);
                narrowest = narrowest.min(cut_model.width);
                cutlist.push(Cut::from(cut_model, model.spacing));
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

    if attempts == 0 {
        cutlist.sort_by(|a, b| a.area().partial_cmp(&b.area()).unwrap());
        if let Some(result) = generate(model, &cutlist, &cut_ranges) {
            results.push(result);
        }
    } else {
        // shuffle approach
        let mut rng = Pcg64::seed_from_u64(12345);

        for attempt in 0..attempts {
            cutlist.shuffle(&mut rng);
            if let Some(result) = generate(model, &cutlist, &cut_ranges) {
                results.push(result);
            }
        }
    }

    if !results.is_empty() {
        let result_count = result_count.min(results.len());

        // sort results by number of boards, increasing, and take the first result_count
        results.sort_by_key(|a| a.len());
        results.truncate(result_count);

        // sort those results by score, decreasing
        results.sort_by(|a, b| score(b).partial_cmp(&score(a)).unwrap());
        println!("Found {} viable solutions", result_count);
        Some(results)
    } else {
        None
    }
}
