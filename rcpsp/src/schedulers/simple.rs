use psp_lib_parser::structs::PspLibProblem;

use crate::{dag::DAG, tabu_list::simple_tabu_list};

pub fn simple_schedule(psp: PspLibProblem, tabu_list_size: u32) {
    let _tabu_list = simple_tabu_list::SimpleTabuList::new(psp.jobs, tabu_list_size as usize);

    let mut dag = DAG::new(&psp);

    // TODO: Create initial solution

    let lower_bound = dag.compute_lower_bound(false);

    dag.graph.reverse();
    let reversed_lower_bound = dag.compute_lower_bound(true);

    dag.graph.reverse();

    let longest_path = dag.find_longest_path();

    println!("lower bounds: {lower_bound:?}");
    println!("reversed lower bounds: {reversed_lower_bound:?}");
    println!("longest_path: {longest_path:?}");

    if let Some(longest_path) = longest_path {
        let longest_path: Vec<u8> = longest_path
            .into_iter()
            .map(|node_index| (node_index.index() + 1) as u8)
            .collect();

        println!("mapped longest_path: {longest_path:?}");
    }

    let job_execution_ranks = dag.compute_job_execution_ranks();

    println!("job_execution_ranks: {job_execution_ranks:?}");
}
