#[derive(Debug, PartialEq)]
pub struct PspLibProblem {
    // file metadata
    pub file_with_basedata: String,
    pub initial_rng: usize,
    // metadata
    pub projects: u8,
    pub jobs: u8,
    pub horizon: u8,
    pub resources: PspLibProblemResources,
    // project info
    pub project_info: Vec<PspLibProjectInformation>,
    // precedence relations
    pub precedence_relations: Vec<PspLibPrecedenceRelation>,
    // requests/duration
    pub request_durations: Vec<PspLibRequestDuration>,
    // resource availabilities
    pub resource_availabilities: PspLibResourceAvailability,
}

#[derive(Debug, PartialEq)]
pub struct PspLibProblemResources {
    pub renewable: u8,
    pub nonrenewable: u8,
    pub doubly_constrained: u8,
}

#[derive(Debug, PartialEq)]
pub struct PspLibProjectInformation {
    pub number: u8,
    pub jobs: u8,
    pub relative_date: u8,
    pub due_date: u8,
    pub tard_cost: u8,
    pub mpm_time: u8,
}

#[derive(Debug, PartialEq)]
pub struct PspLibPrecedenceRelation {
    pub job_number: u8,
    pub mode_count: u8,
    pub successor_count: u8,
    pub successors: Vec<u8>,
}

#[derive(Debug, PartialEq)]
pub struct PspLibRequestDuration {
    pub job_number: u8,
    pub mode: u8,
    pub r1: u8,
    pub r2: u8,
    pub r3: u8,
    pub r4: u8,
}

#[derive(Debug, PartialEq)]
pub struct PspLibResourceAvailability {
    pub r1: u8,
    pub r2: u8,
    pub r3: u8,
    pub r4: u8,
}
