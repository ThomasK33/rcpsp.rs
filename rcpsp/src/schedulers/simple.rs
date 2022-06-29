use log::debug;
use psp_lib_parser::structs::PspLibProblem;

use crate::{
    dag::DAG,
    tabu_list::{simple_tabu_list, TabuList},
};

pub fn simple_schedule(psp: PspLibProblem, tabu_list_size: u32, swap_range: u32) {
    let _tabu_list: Box<dyn TabuList> = Box::new(simple_tabu_list::SimpleTabuList::new(
        psp.jobs,
        tabu_list_size as usize,
    ));

    let dag = DAG::new(&psp);

    // Compute initial solution
    let job_execution_ranks = dag.compute_job_execution_ranks();

    let mut initial_solution = vec![1_u8];
    initial_solution.append(&mut job_execution_ranks.into_iter().flatten().collect());

    // Compute upper bounds
    let upper_bound = dag.compute_upper_bound();
    debug!("upper_bound: {upper_bound}");
    // Compute lower bound
    let lower_bound = dag.compute_lower_bound(false);
    debug!("lower bounds: {lower_bound:?}");

    let schedule: Vec<u8> = dag
        .compute_job_execution_ranks()
        .into_iter()
        .flatten()
        .collect();

    debug!("initial schedule: {schedule:?}");

    let execution_time = dag.compute_execution_time(&schedule);

    debug!("execution_time: {execution_time}");

    let moves = dag.compute_reduced_neighborhood_moves(schedule, swap_range as usize);
    debug!("moves: {moves:?}");

    // TODO: Perform swaps
    // TODO: For each swap reevaluate execution time

    // TODO: Select swap with highest execution time reduction
    // TODO: Check if in tabu list
    // TODO: Potentially add aspiration criteria
}
