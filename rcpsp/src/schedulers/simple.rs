use psp_lib_parser::structs::PspLibProblem;

use crate::{
    dag::DAG,
    tabu_list::{simple_tabu_list, TabuList},
};
use crate::MoveType::Swap;

pub fn simple_schedule(psp: PspLibProblem, tabu_list_size: u32, swap_range: u32) {
    /*
    //not used here
    let _tabu_list: Box<dyn TabuList> = Box::new(simple_tabu_list::SimpleTabuList::new(
        psp.jobs,
        tabu_list_size as usize,
    ));
    */
    let total_job_number:usize = psp.jobs;
    println!("psp.jobs: {total_job_number:?}");

    let dag = DAG::new(&psp);

    // Compute upper bounds
    // let upper_bound = dag.compute_upper_bound();
    // Compute lower bound
    let lower_bound = dag.compute_lower_bound(false);
    println!("lower bounds: {lower_bound:?}");

    // Reverse the graph and compute the lower bound from end to start activity
    // afterwards the graph is reversed
    // dag.graph.reverse();
    // let reversed_lower_bound = dag.compute_lower_bound(true);
    // dag.graph.reverse();

    let longest_path = dag.find_longest_path();

    // println!("longest_path: {longest_path:?}");

    if let Some(longest_path) = longest_path {
        let longest_path: Vec<u8> = longest_path
            .into_iter()
            .map(|node_index| (node_index.index() + 1) as u8)
            .collect();

        println!("mapped longest_path: {longest_path:?}");
    }

    // Compute initial solution
        //adds first job to the ranks
    let mut job_execution_ranks: Vec<Vec<u8>> = vec![vec![1]];
    job_execution_ranks.append(&mut dag.compute_job_execution_ranks());
    let job_execution_ranks = job_execution_ranks;
    //let job_execution_ranks = dag.compute_job_execution_ranks();
    println!("execution_ranks: {job_execution_ranks:?}");

    let initial_solution :Vec<u8> = job_execution_ranks.clone().into_iter().flatten().collect();
    //based on job_numbers, not indices
    //TODO: more initial solutions
    println!("initial_solution: {initial_solution:?}");
    let (new_schedule,schedule_id,schedule_time) =
        improve_schedule(initial_solution.clone(),0, 244, 0,
                     50, swap_range as usize, &dag, tabu_list_size as usize,
                     total_job_number);
    println!("solution: {new_schedule:?}");
    println!("solution: {schedule_time:?}");
    println!("WOW it runs")

}

//whats going on in the threads
fn improve_schedule(
    mut schedule:Vec<u8>,
    _schedule_id:usize,
    mut global_best_solution_time:u8,  //u8 probably to small for the durations
    critical_path_time:u8,         //used for time for consistency
    max_iterations:u16,
    swap_range:usize,
    dag:&DAG,
    tabu_list_size:usize,
    total_job_number:usize, )-> (Vec<u8>,usize,u8){//schedule:Vec<u8>,schedule_number:usize,schedule_time

    //transfer ownership in and out to reduce memory-intitalization time???
    let mut _tabu_list: Box<dyn TabuList> = Box::new(simple_tabu_list::SimpleTabuList::new(
        total_job_number,
        tabu_list_size,
    ));
    let mut best_swap:(u8,u8)=(0,0);
    let mut best_time:u8=255;

    for _iteration in 0 .. max_iterations {
        //still old computation in job#
        let reduced_neighborhood: Vec<(u8,u8)> = dag.compute_reduced_neighborhood_moves(&schedule, swap_range as usize);
        match reduced_neighborhood
            //carry swap as identifier for the time
            .into_iter()

            //evaluate all moves
            //this might not be the most efficient way to do this
            //TODO: try with backtracking style
            //TODO:Dummy Evaluation
            .map( | swap| (swap,dag.evaluate(apply_move( &schedule,swap))))

            //filter for not in tabu list, or global best
            .filter( | (swap, time)| {
                if *time < global_best_solution_time {return true;}
                _tabu_list.is_possible_move(tjn(swap.0,&schedule), tjn(swap.1,&schedule),Swap)
                })
            //get the best of those
            .max_by_key( | (_,time) | *time)
        {
            Some(result) => (best_swap, best_time)= result,
            None => return (schedule, _schedule_id, 0) //no moves possible
        }

        //update schedule
        let temp= (tjn(best_swap.0,&schedule),tjn(best_swap.1,&schedule));
        schedule.swap(temp.0,temp.1);
        println!("schedule: {schedule:?}");
        println!("swapped: {best_swap:?} ; index: {temp:?}");
        //update global_best (only local)
        if best_time<global_best_solution_time{global_best_solution_time=best_time;}
        //update tabu_list
        _tabu_list.add_turn_to_tabu_list(tjn(best_swap.0,&schedule),tjn(best_swap.1,&schedule),Swap);


        if best_time==critical_path_time {break;}
    }
        (schedule,_schedule_id,best_time)
}

//function to translate job_numbers to indices
//very inefficient, cant be avoided
//except by using indices in the first place
//and also have an more efficient way to filter feasible moves
//translate job number
//TODO: FIX THIS BOTTLENECK OF DEATH
fn tjn(x:u8,schedule:&Vec<u8>) -> usize {
    for i in 0..schedule.len(){
        if x== schedule[i] {return i;}
    }
    println!("job# not found tjn");
    7
}
//applies a move to schedule by copy
//swap are still treated as job#, not indices
fn apply_move(schedule:&Vec<u8>, swap:(u8,u8))->Vec<u8>{
    let mut new_schedule:Vec<u8> = schedule.clone();
    new_schedule.swap(tjn(swap.0,&schedule),tjn(swap.1,&schedule));
    new_schedule
}
