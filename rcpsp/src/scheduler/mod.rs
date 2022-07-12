pub mod custom;
pub mod rayon;
pub mod rayon_multi;

#[derive(Debug, Clone)]
pub struct SchedulerOptions {
    pub number_of_iterations: u32,
    pub max_iter_since_best: u32,
    pub tabu_list_size: u32,
    pub swap_range: usize,
    pub parallel: bool,
    pub iter_since_best_reset: Option<u32>,
    pub schedule_count: u32,
}

pub struct OptimizedSchedule {
    pub schedule: Vec<u8>,
    pub duration: usize,
}
