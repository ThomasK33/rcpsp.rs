use psp_lib_parser::structs::PspLibProblem;

use crate::{dag::DAG, tabu_list::simple_tabu_list};

pub fn simple_schedule(psp: PspLibProblem, tabu_list_size: u32, swap_range: u32) {
    let _tabu_list = simple_tabu_list::SimpleTabuList::new(psp.jobs, tabu_list_size as usize);

    let dag = DAG::new(&psp);

    // Compute initial solution
    let job_execution_ranks = dag.compute_job_execution_ranks();

    let mut initial_solution = vec![1 as u8];
    initial_solution.append(&mut job_execution_ranks.into_iter().flatten().collect());

    println!("initial_solution: {initial_solution:?}");

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

    let schedule: Vec<u8> = dag
        .compute_job_execution_ranks()
        .into_iter()
        .flatten()
        .collect();

    let moves = dag.compute_reduced_neighborhood_moves(schedule, swap_range as usize);

    println!("moves: {moves:?}");
}
