pub mod dag;
pub mod schedulers;
pub mod tabu_list;

mod psp_gen;

pub enum MoveType {
    Swap,
    Shift,
    None,
}
