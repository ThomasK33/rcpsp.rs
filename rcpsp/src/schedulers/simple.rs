use log::debug;
use psp_lib_parser::structs::PspLibProblem;

use crate::MoveType::Swap;
use crate::{
    dag::DAG,
    tabu_list::{simple_tabu_list, TabuList},
};

use rand::thread_rng;
use rand::seq::SliceRandom;
use std::sync::{Arc,mpsc};
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Duration;

pub fn simple_schedule(psp: PspLibProblem, tabu_list_size: u32, swap_range: u32) {
    let total_job_number: usize = psp.jobs;
    debug!("psp.jobs: {total_job_number:?}");

    let dag = DAG::new(psp, swap_range as usize);

    // Compute upper bounds
    // let upper_bound = dag.compute_upper_bound();
    // Compute lower bound
    let lower_bound = dag.compute_lower_bound(false);
    debug!("lower bounds: {lower_bound:?}");

    // Reverse the graph and compute the lower bound from end to start activity
    // afterwards the graph is reversed
    // dag.graph.reverse();
    // let reversed_lower_bound = dag.compute_lower_bound(true);
    // dag.graph.reverse();

    let longest_path = dag.find_longest_path();

    // debug!("longest_path: {longest_path:?}");

    if let Some(longest_path) = longest_path {
        let longest_path: Vec<u8> = longest_path
            .into_iter()
            .map(|node_index| (node_index.index() + 1) as u8)
            .collect();

        debug!("mapped longest_path: {longest_path:?}");
    }

    // Compute initial solution
    //adds first job to the ranks
    let mut job_execution_ranks: Vec<Vec<u8>> = vec![vec![1]];
    job_execution_ranks.append(&mut dag.compute_job_execution_ranks());
    let job_execution_ranks = job_execution_ranks;
    //let job_execution_ranks = dag.compute_job_execution_ranks();
    debug!("execution_ranks: {job_execution_ranks:?}");

    let initial_solution: Vec<u8> = job_execution_ranks.clone().into_iter().flatten().collect();

    let thread_count=8;
    let mut solutions: Vec<Vec<u8>> = vec![initial_solution];


    //let mut extra_inital_solutions: Vec<Vec<u8>> = (0..thread_count-1).into_iter().map(|_| job_execution_ranks.clone().into_iter().map(|mut x| x.shuffle(&mut thread_rng())).flatten().collect()).collect();
    let mut extra_inital_solutions: Vec<Vec<u8>> = (0..thread_count-1).into_iter().map(|_| job_execution_ranks.clone().into_iter().flatten().collect()).collect();
    solutions.append(&mut extra_inital_solutions);
    let mut solution_times : Vec<u8> = solutions.clone().into_iter().map(|s| dag.evaluate(s)).collect();
    let mut global_best_solution_time: u8 = solution_times.iter().min().unwrap().clone();
    //based on job_numbers, not indices
    debug!("initial_solution: {solutions:?}");


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
                0,
                swap_range as usize,
                dag_arc,
                tabu_list_size as usize,
                total_job_number,
                id,
                tx_main,
                rx,
            )
        });
        handles.push(handle);
        txs.push(tx);
    }

    //use threads via messages
    for id in 0..thread_count as usize {
        txs[id].send((
            solutions[id].clone(),
            id,
            global_best_solution_time,
            10,
            )
        ).unwrap();
        //thread::sleep(Duration::from_millis(100));
    }

    //manege completed threads
    let mut counter=0;
    //let (new_schedule, schedule_id, schedule_time,id) = rx_main.recv().unwrap();
    for (new_schedule, schedule_id, schedule_time,id) in rx_main {
        println!("Got in main: {},",id);
        println!("solution: {new_schedule:?}");
        println!("schedule_time: {schedule_time:?}, schedule_id:{schedule_id:?}, thread:{id:?}");

        if(schedule_time<=solution_times[schedule_id]){
            solutions[schedule_id]=new_schedule;
            solution_times[schedule_id]=schedule_time;
            if(schedule_time<global_best_solution_time){
                global_best_solution_time=schedule_time
            }
        }
        /*
        { //make this scope more complex!
            txs[id].send((
                solutions[id].clone(),
                id,
                global_best_solution_time,
                10,
            )
            ).unwrap();
        }
        */

        counter+=1;
        if counter>=thread_count{
            break;
        }
    }

    //destroy threads
    for handle in handles {
        handle.join().unwrap();
    }
    println!("WOW it runs")
    /*
    let (new_schedule, schedule_id, schedule_time) = improve_schedule(
        initial_solution.clone(),
        0,
        244,
        50,
        0,
        swap_range as usize,
        &dag,
        tabu_list_size as usize,
        total_job_number,
    );

    debug!("solution: {new_schedule:?}");
    debug!("solution: {schedule_time:?}");
    debug!("WOW it runs")
    */
}

fn thread_body(
    critical_path_time: u8,            //u8 used for time for consistency
    swap_range: usize,
    dag: Arc<DAG>,
    tabu_list_size: usize,
    total_job_number: usize,
    fake_thread_id:usize,
    tx: std::sync::mpsc::Sender<(Vec<u8>, usize, u8, usize)>,//schedule:Vec<u8>,schedule_number:usize,schedule_time,fake_thread_id
    rx: Receiver<(Vec<u8>,usize,u8,u16)>,//schedule:Vec<u8>,_schedule_id: usize, global_best_solution_time: u8, max_iterations: u16,
){
    let (schedule, _schedule_id, global_best_solution_time, max_iterations) = rx.recv().unwrap();
    println!("Got in Thread: ,{}",_schedule_id);

    //dont pass schedule id?
    let (new_schedule, schedule_id, schedule_time) = improve_schedule(
        schedule,
        _schedule_id,
        global_best_solution_time,
        max_iterations,

        0,
        swap_range as usize,
        &dag,
        tabu_list_size as usize,
        total_job_number,
    );

    //for val in vals {
    tx.send((new_schedule, schedule_id, schedule_time, fake_thread_id) ).unwrap();
        //thread::sleep(Duration::from_secs(1));
    //}
}
//can be done by many threads parallel
fn improve_schedule(
    mut schedule: Vec<u8>,
    _schedule_id: usize,
    mut global_best_solution_time: u8, //u8 probably to small for the durations
    max_iterations: u16,

    critical_path_time: u8,            //used for time for consistency
    swap_range: usize,
    dag: &DAG,
    tabu_list_size: usize,
    total_job_number: usize,
) -> (Vec<u8>, usize, u8) {
    //schedule:Vec<u8>,schedule_number:usize,schedule_time

    //transfer ownership in and out to reduce memory- intitalisation time???
    let mut tabu_list: Box<dyn TabuList> = Box::new(simple_tabu_list::SimpleTabuList::new(
        total_job_number,
        tabu_list_size,
    ));
    let mut best_swap: (usize, usize) = (0, 0);
    let mut best_time: u8 = 255;

    for _iteration in 0..max_iterations {
        //still old computation in job#
        let reduced_neighborhood: Vec<(usize, usize)> =
            dag.filtered_reduced_neighborhood(&schedule);
        match reduced_neighborhood
            //carry swap as identifier for the schedule_time
            .into_iter()
            //evaluate all moves
            //this might not be the most efficient way to do this
            //TODO: try with backtracking style
            //TODO: Dummy Evaluation
            .map(|swap| (swap, dag.evaluate(apply_move(&schedule, swap))))
            //filter for not in tabu list, or global best
            .filter(|(swap, time)| {
                if *time < global_best_solution_time {
                    return true;
                }
                tabu_list.is_possible_move(swap.0, swap.1, Swap)
            })
            //get the best of those
            .max_by_key(|(_, time)| *time)
        {
            Some(result) => (best_swap, best_time) = result,
            None => return (schedule, _schedule_id, 0), //no moves possible
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
        //update tabu_list
        tabu_list.add_turn_to_tabu_list(best_swap.0, best_swap.1, Swap);

        if best_time == critical_path_time {
            break;
        }
    }
    (schedule, _schedule_id, best_time)
}

//applies a move to schedule by copy
//swap are still treated as job#, not indices
fn apply_move(schedule: &Vec<u8>, (u, v): (usize, usize)) -> Vec<u8> {
    let mut new_schedule: Vec<u8> = schedule.clone();
    new_schedule.swap(u, v);
    new_schedule
}

/*
pub fn schedule(_schedule: Schedule) -> Result<()> {
    // todo!("Implement scheduler")

    //============ Start Definition of Problem ============
    // number of jobs
    let N= rand::thread_rng().gen_range(8..30);
    //jobs
    let V :Vec<i32> = (0..N).collect();
    // duration of jobs (randomly with 0 for 0th and Nth
    let D :Vec<i32> = (0..N).map(|x| {
            if x==0 || x==N {0}
            else{rand::thread_rng().gen_range(5..25)}
        }).collect();
    //DAG of the Problem
    let E :Vec<Vec<bool>> = (0..N).map(|i| {
            (0..N).map(|j| {
                if i<=j {false} //obere Dreicksmatrix
                else if i==0 || i==N-1 {true} //Anfangs/Endpunkt
                else {
                    let t :i32=rand::thread_rng().gen_range(0..3);
                    if t==0{true}
                    else {false}
                } //chance is 1/3 here for edges
            }).collect()
        }).collect();
    //number of ressources
    let M = rand::thread_rng().gen_range(3..7);
    //maximal resources of system
    let Rmax :Vec<i32> = (0..M).map(|_| rand::thread_rng().gen_range(6..14)).collect();
    //ressource matrix
    let R :Vec<Vec<i32>> = (0..N).map(|_| {
            (0..M).map(|j| {
                //makes lower values where one process wont need more then
                // half the ressources of one type more likely, so sheduling makes more sense
                let t1=rand::thread_rng().gen_range(0..Rmax[j]+1);
                let t2=rand::thread_rng().gen_range(0..Rmax[j]+1);
                t1*t2/Rmax[j]
            }).collect()
        }).collect();
    //============ End Definition of Problem ============


    let delta=rand::thread_rng().gen_range(3..5);
    let mut Nred: Vec<(usize, usize)> = Vec::new();
    for swap_range in 1..delta+1{
        let mut temp : Vec<(usize, usize)>=(0 as usize..(N-swap_range) as usize).map(|i| (i,i+swap_range as usize)).collect();
        Nred.append(&mut temp);
    }
    let tabuList_len=100;

    let InitialSolutions :Vec<Vec<i32>> = vec![(0..N).collect()];
    //because Definition is like that, but there are more initials (4.1.)

    {
        let mut tabuList:Vec<(usize,usize)>=vec![(0,0); tabuList_len];
        let mut tabuCache:Vec<Vec<bool>> = vec![vec![false;N as usize];N as usize];
        let mut writeIndex : usize = 0;
        {
            let Nfeasable: Vec<(usize, usize)> = generate_Nred(&Nred, &InitialSolutions[0], &E);

            if Nfeasable[0]!=(0,0) && !check_STL(Nfeasable[0],&tabuCache){
                addTo_STL(Nfeasable[0], &mut tabuCache, &mut tabuList, &mut writeIndex);
            }
        }
    }



    Ok(())
}

//Algorithm1
fn generate_Nred(Nred: &Vec<(usize, usize)>, sched: &Vec<i32>, edges:&Vec<Vec<bool>>) -> Vec<(usize, usize)> {
    let mut moves: Vec<(usize, usize)>=Nred.clone();
    let moves_len=moves.len();

    ///////
    //removes unfeasible(5)
    for i in 0..moves_len{
        let u=moves[i].0;
        let v=moves[i].1;
        for x in u+1 .. v+1{
            if edges
                [sched[u] as usize]
                [sched[x as usize] as usize]
            {moves[i]=(0,0);break;}
        }
    }
    //moves (0,0) to the end
    let mut i=0;
    let mut invalid=0;
    while i<moves_len-invalid{
        if moves[i]==(0,0){
            while moves[moves_len-invalid-1]==(0,0){
                invalid+=1;
            }
            if moves_len-invalid-1<=i {break;}
            moves.swap(i,moves_len-invalid-1);
            invalid+=1;
        }
        i+=1;
    }
    //removes unfeasible(6)
    for i in 0..moves_len-invalid{
        let u=moves[i].0;
        let v=moves[i].1;
        for x in u .. v{
            if edges
                [sched[x] as usize]
                [sched[v] as usize]
            {moves[i]=(0,0);break;}
        }
    }
    //moves (0,0) to the end
    i=0;
    // invalid=0; //no reset for invalids in the end
    while i<moves_len-invalid{
        if moves[i]==(0,0){
            while moves[moves_len-invalid-1]==(0,0){
                invalid+=1;
            }
            if moves_len-invalid-1<=i {break;}
            moves.swap(i,moves_len-invalid-1);
            invalid+=1;
        }
        i+=1;
    }
    ///////
    ///////
    moves=moves.into_iter().filter(|(u,v)| {
        for x in u+1 .. v+1{
            if edges
                [sched[*u] as usize]
                [sched[x] as usize]
            {return false}
        }
        true
    }).collect();
    moves=moves.into_iter().filter(|(u,v)| {
        for x in *u .. *v{
            if edges
                [sched[x] as usize]
                [sched[*v] as usize]
            {return false}
        }
        true
    }).collect();
    //let t = sched.get(5);
    ///////
    return moves
}

//Algorithm2
fn check_STL((u,v):(usize, usize),tabuCache:&Vec<Vec<bool>>)->bool{
    tabuCache[u][v].clone()
}

//Algorithm3
fn addTo_STL((u,v):(usize, usize), tabuCache:&mut Vec<Vec<bool>>, tabuList:&mut Vec<(usize,usize)>, writeIndex: &mut usize){
    let (uold, vold) = tabuList[*writeIndex].clone();
    tabuCache[uold][vold] = false;
    tabuList[*writeIndex]= (u, v).clone();
    tabuCache[u][v] = true;
    *writeIndex = (*writeIndex + 1) % tabuList.len();

}
 */
