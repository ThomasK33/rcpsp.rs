use crate::MoveType;

pub mod simple_tabu_list;

pub trait TabuList {
    /// Check if move is permitted
    fn is_possible_move(&self, i: usize, j: usize, move_type: MoveType) -> bool;
    /// Add move (specified by i,j,type) to tabu list.
    fn add_turn_to_tabu_list(&mut self, i: usize, j: usize, move_type: MoveType);
    /// Inform tabu list about new best solution.
    fn best_solution_found(&self);
    /// Tell tabu list about end of iteration.
    fn go_to_next_iter(&self) -> usize;
    /// The method removes some tabu moves randomly since all solutions in neighborhood were tabu.
    fn prune(&mut self);
}
