use log::{debug, info, trace};
use psp_lib_parser::structs::PspLibProblem;
use rayon::prelude::*;

use crate::{
    dag::DAG,
    tabu_list::{simple_tabu_list::SimpleTabuList, TabuList},
};

use rand::thread_rng;
use rand::seq::SliceRandom;
use std::sync::{Arc,mpsc};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
//use std::time::Duration;

#[derive(Debug, Clone)]
pub struct SchedulerOptions {
    pub number_of_iterations: u32,
    pub max_iter_since_best: u32,
    pub tabu_list_size: u32,
    pub swap_range: u32,
    pub parallel: bool,
    pub iter_since_best_reset: Option<u32>,
}

pub struct OptimizedSchedule {
    pub schedule: Vec<u8>,
    pub duration: usize,
}

pub fn scheduler(psp: PspLibProblem, mut options: SchedulerOptions) -> OptimizedSchedule {

    //==========settings
    //options.swap_range=10;
    //options.tabu_list_size=40;
    let activity_number: usize=psp.jobs;
    let thread_count : usize={if options.parallel {8} else {1}};
    let schedule_count=thread_count; //using thread_id as schedule_id sometimes

    let improvement_partition = 10;
    let initial_iteration_multiplier = 2;

    let improvement_iterartions = options.number_of_iterations/((thread_count as u32)*improvement_partition);
    let initial_improvement_iterations= improvement_iterartions * initial_iteration_multiplier;
    let improvements=improvement_partition-initial_iteration_multiplier;

    let divesification_iterations=20;

    //==========initialization

    let dag = DAG::new(psp, options.swap_range as usize);//15);

    let lower_bound = dag.compute_lower_bound(false);
    info!("options: {options:?}");
    let lower_bound = lower_bound.map(|lb| lb.0).unwrap_or(0);

    // Compute initial solution
    let mut job_execution_ranks: Vec<Vec<u8>> = vec![vec![1]]; //fixes the missing first job
    job_execution_ranks.append(&mut dag.compute_job_execution_ranks());
    let job_execution_ranks = job_execution_ranks;

    let schedule: Vec<u8> = job_execution_ranks.clone().into_iter().flatten().collect();
    let mut schedules: Vec<Vec<u8>> = vec![schedule];

    let mut extra_inital_solutions: Vec<Vec<u8>> = (0..schedule_count-1).into_iter().map(|_| job_execution_ranks.clone().into_iter().map(|mut x| {x.shuffle(&mut thread_rng());x}).flatten().collect()).collect();
    //let mut extra_inital_solutions: Vec<Vec<u8>> = (0..thread_count-1).into_iter().map(|_| job_execution_ranks.clone().into_iter()                                          .flatten().collect()).collect();
    schedules.append(&mut extra_inital_solutions);
    info!("initial_solution: {schedules:?}");

    let mut schedule_times : Vec<usize> = schedules.iter().map(|s| dag.compute_execution_time(s, None)).collect();
    let mut global_best_solution_time: usize = schedule_times.iter().min().unwrap().clone();
    let mut global_best_solution_schedule: Vec<u8> = schedules.iter().map(|s| (s,dag.compute_execution_time(s, None))).min_by_key(|(_, time)| *time).unwrap().0.clone();

    info!("execution_times: {schedule_times:?}");

    let mut tabu_lists: Vec<SimpleTabuList> = schedule_times.iter().map(|_| SimpleTabuList::new(activity_number, options.tabu_list_size as usize)).collect();
    let empty_tabu=SimpleTabuList::new(0, 0);


    //=================================
    //multi-thread-part
    //=================================

    //spawn threads
    let mut handles = vec![];
    let dag_arc=Arc::new(dag);
    let (tx_main, rx_main) = mpsc::channel();
    let mut txs = vec![];

    for id in 0..thread_count {
        let dag_arc = Arc::clone(&dag_arc);
        let tx_main=tx_main.clone();
        let (tx, rx) = mpsc::channel();
        let handle = thread::spawn(move || {
            thread_body(
                lower_bound,
                dag_arc,
                id,
                tx_main,
                rx,
            )
        });
        handles.push(handle);
        txs.push(tx);
    }

    //initial_improvements
    for id in 0..thread_count {
        let schedule_id=id;
        //using threads via messages
        txs[id].send((
            schedules[schedule_id].clone(),
            schedule_times[schedule_id].clone(),
            id, //using thread_id as schedule_id
            tabu_lists[schedule_id].clone(),
            global_best_solution_time,
            initial_improvement_iterations,
            options.max_iter_since_best,
        )).unwrap();
    }

    //manage running/finished threads
    let mut runnig_threads =thread_count;
    let mut improvements_left=improvements;
    for (new_schedule, schedule_id, new_schedule_time, new_tabu_list, id) in rx_main {
        //let (new_schedule, schedule_id, schedule_time,id) = rx_main.recv().unwrap();
        //message from thread arrived
        debug!("Got in main: {}, schedule_time: {new_schedule_time:?}, schedule_id:{schedule_id:?}, thread:{id:?}, solution: {new_schedule:?} ",id);

        //process message
        if new_schedule_time<=schedule_times[schedule_id] {
            schedules[schedule_id]=new_schedule;
            schedule_times[schedule_id]=new_schedule_time;
            tabu_lists[schedule_id]=new_tabu_list;

            if schedule_times[schedule_id]<global_best_solution_time {
                global_best_solution_time= schedule_times[schedule_id];
                global_best_solution_schedule=schedules[schedule_id].clone();
            }
        }

        //send back new message
        runnig_threads -=1;

        //make this scope more complex! -- done
        if runnig_threads == 0{
            improvements_left-=1;
            if improvements_left<=0 || lower_bound==global_best_solution_time{
                break;
            }
            else{
                //all threads are collected already
                    //ggez

                //sort schedules by time
                let mut indices:Vec<usize> = (0..schedule_count).collect();
                indices.sort_by_key(|x| schedule_times[*x]);

                //take  better half and put in second, ignore center for odd scheule_numbers
                //and diversify all replacing threads
                for i in 0..(schedule_count/2){ //skips center if uneven, or everything if only one
                    let from=indices[i];
                    let to = indices[schedule_count-1-i];

                    schedules[to]=diversify_schedule(
                        schedules[from].clone(),
                        divesification_iterations,
                        &dag_arc
                    );
                    schedule_times[to]=dag_arc.compute_execution_time(&schedules[to], None);
                    tabu_lists[to]=tabu_lists[from].clone();

                }

                //run all threads again
                println!("improvments left: {}",improvements_left);
                for id in 0..thread_count {
                    let schedule_id=id;
                    txs[id].send((
                        schedules[schedule_id].clone(),
                        schedule_times[schedule_id].clone(),
                        schedule_id,
                        tabu_lists[schedule_id].clone(),
                        global_best_solution_time,
                        improvement_iterartions,
                        options.max_iter_since_best,
                    )).unwrap();
                }
                runnig_threads =thread_count;
            }
        }
    }

    //stop all threads with id
    for id in 0..thread_count{
        txs[id].send((vec![],0,id,empty_tabu.clone(),0,0,0,)).unwrap();
    }

    //destroy/join threads
    for handle in handles {
        handle.join().unwrap();
    }
    debug!("WOW it runs");

    //======================
    //end multi-thread-part

    /*
    let schedule_id=0;
    let (best_execution_schedule, schedule_id , best_execution_time,best_tabu_list)=improve_schedule(
        schedules[schedule_id].clone(),
        schedule_times[schedule_id].clone(),
        schedule_id,
        global_best_solution_time,
        options.number_of_iterations,
        options.max_iter_since_best,
        lower_bound,
        &dag,
        tabu_lists[schedule_id].clone()
    );
    */

    info!("best_execution_schedule: {global_best_solution_schedule:?}");
    info!("best_execution_time: {global_best_solution_time}");
    //info!("best_execution_time2: {}",dag.compute_execution_time(&best_execution_schedule, Some(&(1,2))));


    OptimizedSchedule {
        schedule: global_best_solution_schedule,
        duration: global_best_solution_time,
    }
}

fn thread_body(
    lower_bound: usize,            //u8 used for time for consistency
    //swap_range: usize,
    dag: Arc<DAG>,
    //tabu_list_size: usize,
    //total_job_number: usize,
    fake_thread_id:usize,
    tx: Sender<(Vec<u8>, usize, usize, SimpleTabuList, usize)>,//schedule:Vec<u8>,schedule_number:usize,schedule_time,fake_thread_id
    rx: Receiver<(Vec<u8>, usize, usize, SimpleTabuList, usize, u32, u32)>,//schedule,execution_time,_schedule_id,SimpleTabuLis, global_best_solution_time, max_iterations, number_of_iterations,max_iter_since_best
){
    loop{
        let (schedule, schedule_time,_schedule_id, tabu_list, global_best_solution_time, number_of_iterations,max_iter_since_best) = rx.recv().unwrap();
        if number_of_iterations==0{break;}
        debug!("Got in Thread: ,{}",_schedule_id);

        //dont pass schedule id?
        let (new_schedule, schedule_id, new_schedule_time,new_tabu_list)=improve_schedule(
            schedule,
            schedule_time,
            _schedule_id,
            global_best_solution_time,
            number_of_iterations,
            max_iter_since_best,
            lower_bound,
            &dag,
            tabu_list
        );

        tx.send((new_schedule, schedule_id, new_schedule_time, new_tabu_list, fake_thread_id) ).unwrap();
    }
}

fn improve_schedule(
    mut schedule: Vec<u8>,
    schedule_time:usize,
    _schedule_id: usize,
    mut global_best_solution_time: usize, //u8 probably to small for the durations
    max_iterations: u32,
    max_iterations_since_best:u32,

    critical_path_time: usize,            //used for time for consistency
    dag: &DAG,
    mut tabu_list: SimpleTabuList
) -> (Vec<u8>, usize, usize, SimpleTabuList) {
    //schedule:Vec<u8>,schedule_number:usize,schedule_time

    let mut best_swap: &(usize, usize) = &(0, 0);
    let mut best_time: usize = 100000; //value never used

    let mut best_schedule = schedule.clone();
    let mut best_schedule_time= schedule_time;
    let mut last_best_iteration = 0;
    let mut best_tabu_list = tabu_list.clone();

    for _iteration in 0..max_iterations {
        let reduced_neighborhood: Vec<&(usize, usize)> =
            dag.filtered_reduced_neighborhood(&schedule);
        match reduced_neighborhood
            //carry swap as identifier for the schedule_time
            //.into_iter()
            .into_iter()
            //evaluate all moves
            .map(|swap| (swap, dag.compute_execution_time(&schedule, Some(swap))))
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
            None => {println!("this_happened");return (schedule, _schedule_id, 0, best_tabu_list)}, //no moves possible
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
        if best_time < best_schedule_time{
            best_schedule=schedule.clone();
            best_schedule_time=best_time;
            debug!("called by dodo {} since_last {}, in thread/schedule: {}", best_time, _iteration-last_best_iteration, _schedule_id);
            last_best_iteration=_iteration;
            best_tabu_list = tabu_list.clone();
        }
        if (_iteration%(max_iterations/20))==0 {
            debug!("={}%=",5*_iteration/(max_iterations/20));
        }
        if _iteration-last_best_iteration>=max_iterations_since_best{
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
    (best_schedule, _schedule_id, best_schedule_time, best_tabu_list)
}

fn diversify_schedule(
    mut schedule: Vec<u8>,
    iterations: u32,
    dag: &DAG
) -> Vec<u8>{
    for _ in 0..iterations{
        let random_swap=**dag.filtered_reduced_neighborhood(&schedule).choose(&mut thread_rng()).unwrap_or(&&(0,0));
        schedule.swap(random_swap.0, random_swap.1);
    }
    schedule

}

/*
pub fn scheduler(psp: PspLibProblem, options: SchedulerOptions) -> OptimizedSchedule {
    let activity_number: usize=psp.jobs;
    let dag = DAG::new(psp, options.swap_range as usize);

    let lower_bound = dag.compute_lower_bound(false);
    info!("options: {options:?}");
    let lower_bound = lower_bound.map(|lb| lb.0).unwrap_or(0);

    // Compute initial solution
    let mut schedule: Vec<u8> = dag
        .compute_job_execution_ranks()
        .into_iter()
        .flatten()
        .collect();

    info!("initial schedule: {schedule:?}");

    let execution_time = dag.compute_execution_time(&schedule, None);
    info!("execution_time: {execution_time}");

    let mut best_execution_time = execution_time;
    let mut best_execution_schedule = schedule.clone();
    let mut iter_since_best = 0;
    let mut reset_counter = 0;

    // Select swap with highest execution time reduction
    //  Check if in tabu list
    let mut tabu_list = SimpleTabuList::new(activity_number, options.tabu_list_size as usize);
    let mut best_tabu_list = tabu_list.clone();

    for _ in 0..options.number_of_iterations {
        debug!("iter_since_best: {iter_since_best} - best_execution_time: {best_execution_time}");

        if iter_since_best >= options.max_iter_since_best {
            debug!(
                "did not find better move in {iter_since_best} iterations, thus stopping search"
            );
            break;
        }

        if let Some(iter_since_best_reset) = options.iter_since_best_reset {
            if reset_counter >= iter_since_best_reset {
                debug!("did not find a better solution in {reset_counter} iterations, resetting tabu search back to currently best solution");
                schedule = best_execution_schedule.clone();
                reset_counter = 0;
                tabu_list = best_tabu_list.clone();
            }
        }

        let moves = dag.compute_reduced_neighborhood_moves(&schedule, options.swap_range as usize);
        trace!("moves: {moves:?}");

        // Perform swaps and after each swap reevaluate execution time
        let map_op = |(job_a, job_b)| {
            let execution_time = 42;//dag.compute_execution_time(&schedule, Some((job_a, job_b)));

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

        iter_since_best += 1;
        reset_counter += 1;

        if let Some(&highest_rated_move) = rated_moves.first() {
            let (execution_time, (i, j)) = highest_rated_move;

            let index_a = schedule.iter().position(|&job| job == i).unwrap();
            let index_b = schedule.iter().position(|&job| job == j).unwrap();

            schedule.swap(index_a, index_b);

            tabu_list.add_turn_to_tabu_list(i as usize, j as usize);

            if execution_time < best_execution_time {
                best_execution_time = execution_time;
                best_execution_schedule = schedule.clone();
                best_tabu_list = tabu_list.clone();
                iter_since_best = 0;
                reset_counter = 0;
            }
        }

        if best_execution_time == lower_bound {
            // Stop searching once we've found the "physically" best solution
            break;
        }
    }

    info!("best_execution_schedule: {best_execution_schedule:?}");
    info!("best_execution_time: {best_execution_time}");

    OptimizedSchedule {
        schedule: best_execution_schedule,
        duration: best_execution_time,
    }
}
*/
