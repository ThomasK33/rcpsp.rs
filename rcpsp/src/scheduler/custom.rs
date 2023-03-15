use log::{debug, info};
use psp_lib_parser::structs::PspLibProblem;

use crate::{
    dag::DAG,
    tabu_list::{simple_tabu_list::SimpleTabuList, TabuList},
};

use rand::seq::SliceRandom;
use rand::thread_rng;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc};
use std::thread;

use super::{OptimizedSchedule, SchedulerOptions};

pub fn scheduler(psp: PspLibProblem, mut options: SchedulerOptions) -> OptimizedSchedule {
    //==========settings
    options.number_of_iterations = 4000;
    //options.swap_range=10;
    //options.tabu_list_size=40;
    //options.parallel=false;
    let activity_number: usize = psp.jobs;
    let thread_count: usize = {
        if options.parallel {
            num_cpus::get()
        } else {
            1
        }
    };
    let schedule_count = thread_count; //using thread_id as schedule_id sometimes

    let improvement_partition = 10;
    let initial_iteration_multiplier = 2;

    let mut improvement_iterations =
        options.number_of_iterations / ((thread_count as u32) * improvement_partition);
    let mut initial_improvement_iterations = improvement_iterations * initial_iteration_multiplier;
    let mut improvements = improvement_partition - initial_iteration_multiplier;
    let mut max_iter_since_best = initial_improvement_iterations;
    if !options.parallel {
        improvement_iterations = 1;
        initial_improvement_iterations = options.number_of_iterations;
        improvements = 0;
        max_iter_since_best = options.number_of_iterations;
    }

    let diversification_iterations = 20;

    //==========initialization
    let dag = DAG::new(psp, options.swap_range);

    let lower_bound = dag.compute_lower_bound(false);
    info!("lower_bound: {lower_bound:?}");
    info!("options: {options:?}");
    let lower_bound = lower_bound.map(|lb| lb.0).unwrap_or(0);

    // Compute initial solution
    let mut job_execution_ranks: Vec<Vec<u8>> = vec![vec![1]]; //fixes the missing first job
    job_execution_ranks.append(&mut dag.compute_job_execution_ranks());
    let job_execution_ranks = job_execution_ranks;

    let schedule: Vec<u8> = job_execution_ranks.clone().into_iter().flatten().collect();
    let mut schedules: Vec<Vec<u8>> = vec![schedule];

    let mut extra_initial_solutions: Vec<Vec<u8>> = (0..schedule_count - 1)
        .map(|_| {
            job_execution_ranks
                .clone()
                .into_iter()
                .flat_map(|mut x| {
                    x.shuffle(&mut thread_rng());
                    x
                })
                .collect()
        })
        .collect();
    //let mut extra_initial_solutions: Vec<Vec<u8>> = (0..thread_count-1).into_iter().map(|_| job_execution_ranks.clone().into_iter()                                          .flatten().collect()).collect();
    schedules.append(&mut extra_initial_solutions);
    info!("initial_solution: {schedules:?}");

    let mut schedule_times: Vec<usize> = schedules
        .iter()
        .map(|s| dag.compute_execution_time(s, None))
        .collect();
    let mut global_best_solution_time: usize = *schedule_times.iter().min().unwrap();
    let mut global_best_solution_schedule: Vec<u8> = schedules
        .iter()
        .map(|s| (s, dag.compute_execution_time(s, None)))
        .min_by_key(|(_, time)| *time)
        .unwrap()
        .0
        .clone();

    info!("execution_times: {schedule_times:?}");

    let mut tabu_lists: Vec<SimpleTabuList> = schedule_times
        .iter()
        .map(|_| SimpleTabuList::new(activity_number, options.tabu_list_size as usize))
        .collect();
    let empty_tabu = SimpleTabuList::new(0, 0);

    //=================================
    //multi-thread-part
    //=================================

    //spawn threads
    let mut handles = vec![];
    let dag_arc = Arc::new(dag);
    let (tx_main, rx_main) = mpsc::channel();
    let mut txs = vec![];

    for id in 0..thread_count {
        let dag_arc = Arc::clone(&dag_arc);
        let tx_main = tx_main.clone();
        let (tx, rx) = mpsc::channel();
        let handle = thread::spawn(move || thread_body(lower_bound, dag_arc, id, tx_main, rx));
        handles.push(handle);
        txs.push(tx);
    }

    //initial_improvements
    for (id, txs_id) in txs.iter().enumerate().take(thread_count) {
        //using threads via messages
        txs_id
            .send(ThreadInfo {
                schedule: schedules[id].clone(),
                schedule_time: schedule_times[id],
                schedule_id: id, //using thread_id as schedule_id
                tabu_list: tabu_lists[id].clone(),
                global_best_solution_time,
                number_of_iterations: initial_improvement_iterations,
                max_iter_since_best,
            })
            .unwrap();
    }

    //manage running/finished threads
    let mut running_threads = thread_count;
    let mut improvements_left = improvements;
    for ThreadData {
        new_schedule,
        schedule_id,
        new_schedule_time,
        new_tabu_list,
        id,
    } in rx_main
    {
        //let (new_schedule, schedule_id, schedule_time,id) = rx_main.recv().unwrap();
        //message from thread arrived
        debug!("Got in main: {}, schedule_time: {new_schedule_time:?}, schedule_id:{schedule_id:?}, thread:{id:?}, solution: {new_schedule:?} ",id);

        //process message
        if new_schedule_time <= schedule_times[schedule_id] {
            schedules[schedule_id] = new_schedule;
            schedule_times[schedule_id] = new_schedule_time;
            tabu_lists[schedule_id] = new_tabu_list;

            if schedule_times[schedule_id] < global_best_solution_time {
                global_best_solution_time = schedule_times[schedule_id];
                global_best_solution_schedule = schedules[schedule_id].clone();
            }
        }

        //send back new message
        running_threads -= 1;

        //make this scope more complex! -- done
        if running_threads == 0 {
            if improvements_left == 0 || lower_bound == global_best_solution_time {
                break;
            } else {
                //all threads are collected already
                //ggez

                //sort schedules by time
                let mut indices: Vec<usize> = (0..schedule_count).collect();
                indices.sort_by_key(|x| schedule_times[*x]);

                //take  better half and put in second, ignore center for odd schedule_numbers
                //and diversify all replacing threads
                for i in 0..(schedule_count / 2) {
                    //skips center if uneven, or everything if only one
                    let from = indices[i];
                    let to = indices[schedule_count - 1 - i];

                    schedules[to] = diversify_schedule(
                        schedules[from].clone(),
                        diversification_iterations,
                        &dag_arc,
                    );
                    schedule_times[to] = dag_arc.compute_execution_time(&schedules[to], None);
                    tabu_lists[to] = tabu_lists[from].clone();
                }

                //run all threads again
                debug!("improvements left: {}", improvements_left);
                for (id, txs_id) in txs.iter().enumerate().take(thread_count) {
                    txs_id
                        .send(ThreadInfo {
                            schedule: schedules[id].clone(),
                            schedule_time: schedule_times[id],
                            schedule_id: id,
                            tabu_list: tabu_lists[id].clone(),
                            global_best_solution_time,
                            number_of_iterations: improvement_iterations,
                            max_iter_since_best,
                        })
                        .unwrap();
                }

                running_threads = thread_count;
            }
            improvements_left -= 1;
        }
    }

    //stop all threads with id
    for (id, txs_id) in txs.iter().enumerate().take(thread_count) {
        txs_id
            .send(ThreadInfo {
                schedule: vec![],
                schedule_time: 0,
                schedule_id: id,
                tabu_list: empty_tabu.clone(),
                global_best_solution_time: 0,
                number_of_iterations: 0,
                max_iter_since_best: 0,
            })
            .unwrap();
    }

    //destroy/join threads
    for handle in handles {
        handle.join().unwrap();
    }
    debug!("WOW it runs");

    //======================
    //end multi-thread-part

    info!("best_execution_schedule: {global_best_solution_schedule:?}");
    info!("best_execution_time: {global_best_solution_time}");
    //info!("best_execution_time2: {}",dag.compute_execution_time(&best_execution_schedule, Some(&(1,2))));

    OptimizedSchedule {
        schedule: global_best_solution_schedule,
        duration: global_best_solution_time,
    }
}

struct ThreadInfo {
    schedule: Vec<u8>,
    schedule_time: usize,
    schedule_id: usize,
    tabu_list: SimpleTabuList,
    global_best_solution_time: usize,
    number_of_iterations: u32,
    max_iter_since_best: u32,
}

struct ThreadData {
    new_schedule: Vec<u8>,
    schedule_id: usize,
    new_schedule_time: usize,
    new_tabu_list: SimpleTabuList,
    id: usize,
}

fn thread_body(
    lower_bound: usize,
    dag: Arc<DAG>,
    fake_thread_id: usize,
    tx: Sender<ThreadData>,
    rx: Receiver<ThreadInfo>,
) {
    loop {
        let ThreadInfo {
            schedule,
            schedule_time,
            schedule_id,
            tabu_list,
            global_best_solution_time,
            number_of_iterations,
            max_iter_since_best,
        } = rx.recv().unwrap();
        if number_of_iterations == 0 {
            break;
        }
        debug!("Got in Thread: ,{}", schedule_id);

        //don't pass schedule id?
        let (new_schedule, schedule_id, new_schedule_time, new_tabu_list) =
            improve_schedule(ImproveScheduleArguments {
                schedule,
                schedule_time,
                schedule_id,
                global_best_solution_time,
                max_iterations: number_of_iterations,
                max_iterations_since_best: max_iter_since_best,
                critical_path_time: lower_bound,
                dag: &dag,
                tabu_list,
            });

        tx.send(ThreadData {
            new_schedule,
            schedule_id,
            new_schedule_time,
            new_tabu_list,
            id: fake_thread_id,
        })
        .unwrap();
    }
}

struct ImproveScheduleArguments<'a> {
    schedule: Vec<u8>,
    schedule_time: usize,
    schedule_id: usize,
    global_best_solution_time: usize, //u8 probably to small for the durations
    max_iterations: u32,
    max_iterations_since_best: u32,

    critical_path_time: usize, //used for time for consistency
    dag: &'a DAG,
    tabu_list: SimpleTabuList,
}

fn improve_schedule(args: ImproveScheduleArguments) -> (Vec<u8>, usize, usize, SimpleTabuList) {
    //schedule:Vec<u8>,schedule_number:usize,schedule_time

    let ImproveScheduleArguments {
        mut schedule,
        schedule_time,
        schedule_id,
        mut global_best_solution_time,
        max_iterations,
        max_iterations_since_best,
        critical_path_time,
        dag,
        mut tabu_list,
    } = args;

    let mut best_swap: &(usize, usize);
    let mut best_time: usize; //value never used

    let mut best_schedule = schedule.clone();
    let mut best_schedule_time = schedule_time;
    let mut last_best_iteration = 0;
    let mut best_tabu_list = tabu_list.clone();

    for _iteration in 0..max_iterations {
        let reduced_neighborhood: Vec<&(usize, usize)> =
            dag.filtered_reduced_neighborhood(&schedule);
        match reduced_neighborhood
            //carry swap as identifier for the schedule_time
            .into_iter()
            //evaluate all moves
            .map(|swap| {
                (
                    swap,
                    dag.compute_execution_time(
                        &schedule,
                        Some((schedule[swap.0], schedule[swap.1])),
                    ),
                )
            })
            //filter for not in tabu list, or global best
            .filter(|(swap, time)| {
                if *time < global_best_solution_time {
                    return true;
                }
                tabu_list.is_possible_move(swap.0, swap.1)
            })
            //get the best of those
            .min_by_key(|(_, time)| *time)
        {
            Some(result) => (best_swap, best_time) = result,
            None => {
                debug!("this_happened");
                return (
                    best_schedule,
                    schedule_id,
                    best_schedule_time,
                    best_tabu_list,
                );
            } //no moves possible
        }

        //update schedule
        schedule.swap(best_swap.0, best_swap.1);
        debug!("schedule: {schedule:?}");
        let temp = (schedule[best_swap.0], schedule[best_swap.1]);
        debug!("swapped: {temp:?} ; index: {best_swap:?}");
        //update global_best (only local)
        if best_time < global_best_solution_time {
            global_best_solution_time = best_time;
        }
        if best_time <= best_schedule_time {
            best_schedule = schedule.clone();
            best_schedule_time = best_time;
            debug!(
                "called by dodo {} since_last {}, in thread/schedule: {}",
                best_time,
                _iteration - last_best_iteration,
                schedule_id
            );
            last_best_iteration = _iteration;
            best_tabu_list = tabu_list.clone();
        }
        //if (_iteration%(max_iterations/20))==0 {
        //    debug!("={}%=",5*_iteration/(max_iterations/20));
        //}
        if _iteration - last_best_iteration >= max_iterations_since_best {
            debug!(
                "did not find better move in {max_iterations_since_best} iterations, thus stopping search"
            );
            break;
        }
        //update tabu_list
        tabu_list.add_turn_to_tabu_list(best_swap.0, best_swap.1);

        if best_time == critical_path_time {
            debug!("Critical Hit");
            break;
        }
    }
    (
        best_schedule,
        schedule_id,
        best_schedule_time,
        best_tabu_list,
    )
}

fn diversify_schedule(mut schedule: Vec<u8>, iterations: u32, dag: &DAG) -> Vec<u8> {
    for _ in 0..iterations {
        let random_swap = **dag
            .filtered_reduced_neighborhood(&schedule)
            .choose(&mut thread_rng())
            .unwrap_or(&&(0, 0));
        schedule.swap(random_swap.0, random_swap.1);
    }
    schedule
}
