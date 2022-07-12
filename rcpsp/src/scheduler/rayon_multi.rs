use log::{debug, info};
use psp_lib_parser::structs::PspLibProblem;
use rand::{prelude::SliceRandom, thread_rng};
use rayon::prelude::*;

use crate::{
    dag::DAG,
    tabu_list::{simple_tabu_list::SimpleTabuList, TabuList},
};

use super::{OptimizedSchedule, SchedulerOptions};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct InterimSchedule {
    pub schedule: Vec<u8>,

    pub best_duration: usize,
    pub best_schedule: Vec<u8>,

    pub tabu_list: SimpleTabuList,
}

type RatedMove = Vec<(usize, (u8, u8))>;

pub fn scheduler(psp: PspLibProblem, options: SchedulerOptions) -> OptimizedSchedule {
    let dag = DAG::new(psp.clone(), options.swap_range);

    let lower_bound = dag.compute_lower_bound(false);
    info!("lower bound: {lower_bound:?}");
    let lower_bound = lower_bound.map(|lb| lb.0).unwrap_or(0);

    // Compute initial solutions
    let mut schedules: Vec<InterimSchedule> = Vec::with_capacity(options.schedule_count as usize);

    let schedule: Vec<u8> = dag
        .compute_job_execution_ranks()
        .into_iter()
        .flatten()
        .collect();
    schedules.push(InterimSchedule {
        schedule,
        best_duration: usize::MAX,
        best_schedule: vec![],
        tabu_list: SimpleTabuList::new(psp.jobs, options.tabu_list_size as usize),
    });

    let job_execution_ranks = dag.compute_job_execution_ranks();
    for _ in 1..options.schedule_count {
        let schedule: Vec<u8> = job_execution_ranks
            .clone()
            .into_iter()
            .flat_map(|mut x| {
                x.shuffle(&mut thread_rng());
                x
            })
            .collect();
        schedules.push(InterimSchedule {
            schedule,
            best_duration: usize::MAX,
            best_schedule: vec![],
            tabu_list: SimpleTabuList::new(psp.jobs, options.tabu_list_size as usize),
        });
    }

    let mut best_global_duration = usize::MAX;
    let mut iter_since_best = 0;

    // Select swap with highest execution time reduction
    //  Check if in tabu list

    for _ in 0..(options.number_of_iterations / options.schedule_count) {
        debug!("iter_since_best: {iter_since_best} - best_global_duration: {best_global_duration}");

        if iter_since_best >= options.max_iter_since_best {
            debug!(
                "did not find better move in {iter_since_best} iterations, thus stopping search"
            );
            break;
        }

        // Perform swaps and after each swap reevaluate execution time
        let map_op = |schedule: Vec<u8>, (job_a, job_b)| {
            let execution_time = dag.compute_execution_time(&*schedule, Some((job_a, job_b)));

            (execution_time, (job_a, job_b))
        };
        let filter_op = |local_tabu_list: SimpleTabuList,
                         (execution_time, (i, j)): &(usize, (u8, u8))| {
            local_tabu_list.is_possible_move(*i as usize, *j as usize)
                || *execution_time < best_global_duration
        };

        let rated_moves_and_schedule: Vec<(RatedMove, &mut InterimSchedule)> = if options.parallel {
            schedules
                .par_iter_mut()
                .map(|interim_schedule| {
                    (
                        dag.compute_reduced_neighborhood_moves(
                            &interim_schedule.schedule,
                            options.swap_range,
                        ),
                        interim_schedule,
                    )
                })
                .map(|(feasible_moves, interim_schedule)| {
                    let mut processed_moves: Vec<(usize, (u8, u8))> = feasible_moves
                        .into_iter()
                        .map(|possible_move| {
                            map_op(interim_schedule.schedule.clone(), possible_move)
                        })
                        .filter(|value| filter_op(interim_schedule.tabu_list.clone(), value))
                        .collect();

                    processed_moves.sort_by_key(|(duration, _)| *duration);

                    (processed_moves, interim_schedule)
                })
                .collect()
        } else {
            schedules
                .iter_mut()
                .map(|interim_schedule| {
                    (
                        dag.compute_reduced_neighborhood_moves(
                            &interim_schedule.schedule,
                            options.swap_range,
                        ),
                        interim_schedule,
                    )
                })
                .map(|(feasible_moves, interim_schedule)| {
                    let mut processed_moves: Vec<(usize, (u8, u8))> = feasible_moves
                        .into_iter()
                        .map(|possible_move| {
                            map_op(interim_schedule.schedule.clone(), possible_move)
                        })
                        .filter(|value| filter_op(interim_schedule.tabu_list.clone(), value))
                        .collect();

                    processed_moves.sort_by_key(|(duration, _)| *duration);

                    (processed_moves, interim_schedule)
                })
                .collect()
        };

        for (rated_moves, interim_schedule) in rated_moves_and_schedule {
            let local_best_duration = interim_schedule.best_duration;

            if let Some(&(duration, (i, j))) = rated_moves.get(0) {
                let a = interim_schedule
                    .schedule
                    .iter()
                    .position(|&job| job == i)
                    .unwrap();
                let b = interim_schedule
                    .schedule
                    .iter()
                    .position(|&job| job == j)
                    .unwrap();

                interim_schedule.schedule.swap(a, b);
                interim_schedule
                    .tabu_list
                    .add_turn_to_tabu_list(i as usize, j as usize);

                if duration < local_best_duration {
                    interim_schedule.best_duration = duration;
                    interim_schedule.best_schedule = interim_schedule.schedule.clone();
                }
            }
        }

        iter_since_best += 1;

        for schedule in &schedules {
            if schedule.best_duration < best_global_duration {
                best_global_duration = schedule.best_duration;
                iter_since_best = 0;
            }
        }

        if best_global_duration == lower_bound {
            // Stop searching once we've found the "physically" best solution
            break;
        }
    }

    schedules.sort_by_key(|schedule| schedule.best_duration);
    let best_execution_schedule = schedules[0].best_schedule.clone();

    info!("best_execution_schedule: {best_execution_schedule:?}");
    info!("best_global_duration: {best_global_duration}");

    OptimizedSchedule {
        schedule: best_execution_schedule,
        duration: best_global_duration,
    }
}
