use log::{debug, info, trace};
use psp_lib_parser::structs::PspLibProblem;
use rayon::prelude::*;

use crate::{
    dag::DAG,
    tabu_list::{simple_tabu_list::SimpleTabuList, TabuList},
};

#[derive(Debug, Clone)]
pub struct SchedulerOptions {
    pub number_of_iterations: u32,
    pub max_iter_since_best: u32,
    pub tabu_list_size: u32,
    pub swap_range: u32,
    pub parallel: bool,
    pub iter_since_best_reset: u32,
}

pub fn scheduler(psp: PspLibProblem, options: SchedulerOptions) {
    let dag = DAG::new(&psp);

    info!("lower bound: {:?}", dag.compute_lower_bound(false));

    // Compute initial solution
    let mut schedule: Vec<u8> = dag
        .compute_job_execution_ranks()
        .into_iter()
        .flatten()
        .collect();

    info!("initial schedule: {schedule:?}");

    let execution_time = dag.compute_execution_time(&schedule);
    info!("execution_time: {execution_time}");

    let mut best_execution_time = execution_time;
    let mut best_execution_schedule = schedule.clone();
    let mut iter_since_best = 0;
    let mut reset_counter = 0;

    // Select swap with highest execution time reduction
    //  Check if in tabu list
    let mut tabu_list = SimpleTabuList::new(psp.jobs, options.tabu_list_size as usize);

    for _ in 0..options.number_of_iterations {
        debug!("iter_since_best: {iter_since_best}");

        if iter_since_best >= options.max_iter_since_best {
            debug!(
                "did not find better move in {iter_since_best} iterations, thus stopping search"
            );
            break;
        }

        if reset_counter >= options.iter_since_best_reset {
            debug!("did not find a better solution in {reset_counter} iterations, resetting tabu search back to currently best solution");
            schedule = best_execution_schedule.clone();
            reset_counter = 0;
            tabu_list.prune();
        }

        let moves = dag.compute_reduced_neighborhood_moves(&schedule, options.swap_range as usize);
        trace!("moves: {moves:?}");

        // Perform swaps and after each swap reevaluate execution time
        let map_op = |(job_a, job_b)| {
            // Swap positions in slice
            let schedule = schedule.clone();

            let index_a = schedule.iter().position(|&job| job == job_a).unwrap();
            let index_b = schedule.iter().position(|&job| job == job_b).unwrap();

            let mut schedule = schedule;
            schedule.swap(index_a, index_b);

            let execution_time = dag.compute_execution_time(&schedule);

            (execution_time, (job_a, job_b))
        };
        let filter_op = |(execution_time, (i, j)): &(usize, (u8, u8))| {
            tabu_list.is_possible_move(*i as usize, *j as usize)
                || *execution_time < best_execution_time
        };

        let mut rated_moves: Vec<(usize, (u8, u8))> = {
            if options.parallel {
                moves
                    .into_par_iter()
                    .map(map_op)
                    .filter(filter_op)
                    .collect()
            } else {
                moves.into_iter().map(map_op).filter(filter_op).collect()
            }
        };
        rated_moves.sort_by_key(|evaluated_move| evaluated_move.0);
        trace!("rated_moves: {rated_moves:?}");

        if let Some(&highest_rated_move) = rated_moves.first() {
            let (execution_time, (i, j)) = highest_rated_move;

            let index_a = schedule.iter().position(|&job| job == i).unwrap();
            let index_b = schedule.iter().position(|&job| job == j).unwrap();

            schedule.swap(index_a, index_b);

            tabu_list.add_turn_to_tabu_list(i as usize, j as usize);

            if execution_time < best_execution_time {
                best_execution_time = execution_time;
                best_execution_schedule = schedule.clone();
                iter_since_best = 0;
                reset_counter = 0;
            }
        }

        iter_since_best += 1;
        reset_counter += 1;
    }

    info!("best_execution_schedule: {best_execution_schedule:?}");
    info!("best_execution_time: {best_execution_time}");
}
