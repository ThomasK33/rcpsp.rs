pub mod capacity_resolution;
pub mod time_resolution;

pub use capacity_resolution::CapacityResolution;
pub use time_resolution::TimeResolution;

pub enum EvaluationAlgorithm {
    CapacityResolution,
    TimeResolution,
}

pub trait SourcesLoad {
    /// It finds out the earliest possible activity start time without resource overload.
    fn get_earliest_start_time(
        &self,
        activity_resource_requirements: &[usize],
        earliest_precedence_start_time: usize,
        activity_duration: usize,
    ) -> usize;

    /// It updates state of resources with respect to the added activity.
    fn add_activity(
        &mut self,
        activity_start: usize,
        activity_stop: usize,
        activity_requirements: &[usize],
    );
}
