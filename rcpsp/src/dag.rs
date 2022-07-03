use hashbrown::{HashMap, HashSet};

use log::trace;
use petgraph::algo;
use petgraph::visit::NodeIndexable;
use psp_lib_parser::structs::PspLibProblem;

// type Graph = petgraph::Graph<u8, u8>;
// type NodeId = petgraph::graph::NodeIndex;
type Graph = petgraph::matrix_graph::MatrixGraph<u8, u8>;
type NodeId = petgraph::matrix_graph::NodeIndex;

pub struct DAG<'a> {
    durations: HashMap<NodeId, u8>,
    graph: Graph,
    job_to_nodes: HashMap<u8, NodeId>,
    node_to_jobs: HashMap<NodeId, u8>,
    pub psp: &'a PspLibProblem,

    requests: HashMap<u8, Vec<u8>>,
}

impl<'a> DAG<'a> {
    pub fn new(psp: &'a PspLibProblem) -> Self {
        // let mut graph = petgraph::graph::DiGraph::<u8, u8>::new();
        let mut graph = petgraph::matrix_graph::DiMatrix::<u8, u8>::new();

        let mut job_to_nodes = HashMap::new();
        let mut node_to_jobs = HashMap::new();
        let mut durations = HashMap::new();
        let mut requests = HashMap::new();

        for duration in &psp.request_durations {
            let node = graph.add_node(duration.duration);
            job_to_nodes.insert(duration.job_number, node);
            node_to_jobs.insert(node, duration.job_number);
            durations.insert(node, duration.duration);
        }

        for relation in &psp.precedence_relations {
            for successor in &relation.successors {
                if let Some(job_index) = job_to_nodes.get(&(relation.job_number)) {
                    if let Some(successor_index) = job_to_nodes.get(successor) {
                        graph.add_edge(*job_index, *successor_index, 0);
                    }
                }
            }
        }

        for request in &psp.request_durations {
            requests.insert(
                request.job_number,
                // (request.r1, request.r2, request.r3, request.r4),
                vec![request.r1, request.r2, request.r3, request.r4],
            );
        }

        Self {
            durations,
            graph,
            job_to_nodes,
            node_to_jobs,
            psp,
            requests,
        }
    }

    /// Compute the upper bound of execution time by accumulating all durations
    pub fn compute_upper_bound(&self) -> usize {
        self.psp
            .request_durations
            .iter()
            .fold(0, |acc, duration| acc + (duration.duration as usize))
    }
    /// Find the lower bound of execution time, based on the longest time in the graph
    pub fn compute_lower_bound(&self, reversed: bool) -> Option<(u8, Vec<NodeId>)> {
        let from = if !reversed {
            self.graph.from_index(0)
        } else {
            self.graph.from_index(self.psp.jobs - 1)
        };
        let to = if !reversed {
            self.graph.from_index(self.psp.jobs - 1)
        } else {
            self.graph.from_index(0)
        };

        algo::all_simple_paths::<Vec<_>, _>(&self.graph, from, to, 0, None)
            .map(|path| {
                let weight = path.iter().fold(0, |acc, node_index| {
                    if let Some(weight) = self.durations.get(node_index) {
                        acc + *weight
                    } else {
                        acc
                    }
                });

                (weight, path)
            })
            .max_by_key(|path| path.0)
    }

    /// Based on the number of edges
    pub fn find_longest_path(&self) -> Option<Vec<NodeId>> {
        algo::all_simple_paths::<Vec<_>, _>(
            &self.graph,
            self.graph.from_index(0),
            self.graph.from_index(self.psp.jobs - 1),
            0,
            None,
        )
        .map(|path| (path.len(), path))
        .max_by_key(|path| path.0)
        .map(|(_, path)| path)
    }

    /// Returns vector of job number execution ranks.
    ///
    /// Warning: those ranks are not node ids but job ids
    pub fn compute_job_execution_ranks(&self) -> Vec<Vec<u8>> {
        let mut successor_map = {
            let mut map = HashMap::new();
            for node in &self.psp.precedence_relations {
                map.insert(node.job_number, node.successors.clone());
            }
            map
        };

        let prerequisite_map: HashMap<u8, Vec<u8>> = {
            let mut map: HashMap<u8, Vec<u8>> = HashMap::new();
            for (job_number, successors) in successor_map.iter() {
                for successor in successors {
                    if let Some(requirements) = map.get_mut(successor) {
                        requirements.push(*job_number);
                    } else {
                        map.insert(*successor, vec![*job_number]);
                    }
                }
            }
            map
        };

        let mut ranks: Vec<Vec<u8>> = vec![];

        // Initially get all successors from the first node
        let mut same_rank = successor_map.remove(&1).unwrap_or_default();

        let mut visited_nodes: HashSet<u8> = HashSet::new();
        loop {
            if same_rank.is_empty() {
                break;
            }

            for job in same_rank.iter() {
                visited_nodes.insert(*job);
            }

            // Expand all successors of current same_level nodes
            let successors: Vec<Vec<u8>> = same_rank
                .iter()
                .map(|current_job| {
                    successor_map
                        .remove(current_job)
                        .unwrap_or_default()
                        .into_iter()
                        // Get all pre requisites and check if they have already been visited
                        .filter(|k| {
                            if let Some(requirements) = prerequisite_map.get(k) {
                                requirements.iter().all(|k| visited_nodes.contains(k))
                            } else {
                                true
                            }
                        })
                        .collect()
                })
                .collect();

            // Push current level to ranks
            ranks.push(same_rank);

            // Replace same_level with next successors
            same_rank = successors.into_iter().flatten().collect::<Vec<u8>>();
            same_rank.sort_unstable();
            same_rank.dedup();
        }

        ranks
    }

    pub fn compute_reduced_neighborhood_moves(
        &self,
        schedule: &[u8],
        swap_range: usize,
    ) -> Vec<(u8, u8)> {
        // Filter out infeasible moves, i.e. moves that violate a precedence relation
        let filter_op = |(u, v): &(u8, u8)| {
            // Check paths not existing from u to v and check paths not existing from u,
            // u+1, u+2, ..., v-1 to v
            let index_u = schedule.iter().position(|&node| node == *u);
            let index_v = schedule.iter().position(|&node| node == *v);

            if let Some(index_u) = index_u {
                if let Some(index_v) = index_v {
                    let start_index = index_u.min(index_v);
                    let end_index = index_v.max(index_u);

                    let nodes_between = &schedule[start_index..end_index + 1];

                    return nodes_between.iter().all(|node| {
                        !(self.graph.has_edge(
                            self.job_to_nodes[node],
                            self.job_to_nodes[nodes_between.last().unwrap()],
                        ) || self.graph.has_edge(
                            self.job_to_nodes[nodes_between.first().unwrap()],
                            self.job_to_nodes[node],
                        ))
                    });
                }
            }

            false
        };

        // Reduced neighborhood to initial solution depends on the neighborhood size
        // parameter (swap range) and is an upper bound for move generation
        let windows: Vec<&[u8]> = schedule.windows(swap_range).collect();

        if let Some((&last_window, windows)) = windows.split_last() {
            let all_moves = windows.iter().map_while(|&window: &&[u8]| {
                if let Some(first) = window.first() {
                    let neighbors: Vec<(u8, u8)> = window
                        .iter()
                        .skip(1)
                        .map(|neighbor| (*first.min(neighbor), *neighbor.max(first)))
                        .filter(filter_op)
                        .collect();

                    return Some(neighbors);
                }

                None
            });

            let mut last_window = last_window.to_vec();
            last_window.append(&mut vec![0; swap_range as usize]);

            let last_window_moves = last_window.windows(swap_range).map_while(|window| {
                if let Some(first) = window.first() {
                    let neighbors: Vec<(u8, u8)> = window
                        .iter()
                        .skip(1)
                        .filter(|&neighbor| *neighbor != 0)
                        .map(|neighbor| (*first.min(neighbor), *neighbor.max(first)))
                        .filter(filter_op)
                        .collect();

                    return Some(neighbors);
                }

                None
            });

            all_moves.chain(last_window_moves).flatten().collect()
        } else {
            vec![]
        }
    }

    pub fn compute_execution_time(&self, schedule: &[u8], swap: Option<(u8, u8)>) -> usize {
        let mut resources: Vec<Vec<u32>> = vec![vec![0; self.compute_upper_bound()]; 4];
        let resource_limits = vec![
            self.psp.resource_availabilities.r1,
            self.psp.resource_availabilities.r2,
            self.psp.resource_availabilities.r3,
            self.psp.resource_availabilities.r4,
        ];

        // Mapping of job number --> earliest job start time
        let mut start_times: HashMap<u8, usize> = HashMap::new();

        // Insert the genesis task with a start time of 0
        start_times.insert(1, 0);

        // Compute earliest start time for each task
        // The earliest start time for a job is: maximum(start time of all it's predecessors + their execution time)

        for job_id in schedule {
            let job_id = &if let Some((i, j)) = swap {
                if *job_id == i {
                    j
                } else if *job_id == j {
                    i
                } else {
                    *job_id
                }
            } else {
                *job_id
            };

            let predecessors_node_ids = self.graph.neighbors_directed(
                *self.job_to_nodes.get(job_id).unwrap(),
                petgraph::EdgeDirection::Incoming,
            );

            let start_time = predecessors_node_ids
                .map(|node_id| *self.node_to_jobs.get(&node_id).unwrap())
                .map(|job_number| {
                    let duration = self
                        .durations
                        .get(self.job_to_nodes.get(&job_number).unwrap())
                        .copied()
                        .unwrap_or(0) as usize;

                    *start_times.get(&job_number).unwrap_or(&0) + duration
                })
                .max();

            if let Some(start_time) = start_time {
                let mut start_time = start_time;

                // Once the earliest start time has been determined, try fitting the task into the resources vector
                if let Some(requirements) = self.requests.get(job_id) {
                    // (0..4).into_iter();

                    // (1) For each index in 0..4:
                    // (2) - check if: resources[index][start_time] + requirements[index] <= resource_limits[index]
                    // (3) - if true:
                    // (4) --> for d in 0..self.durations[job_id]:
                    // (5)       - check if resources[index][start_time + d] + requirements[index] <= resource_limits[index]
                    // (6)       - if true: continue
                    // (7)       - else: start_time += 1 --> repeat (1)
                    // (8)     - loop (4) finishes, continue loop (2) iteration
                    // (9) - loop (2) finishes --> finish
                    // (4) - else --> start_time += 1 --> repeat (1)

                    loop {
                        let mut finished = true;

                        'index_loop: for index in 0..4 {
                            for duration in 0..self.durations[&self.job_to_nodes[job_id]] {
                                if resources[index][start_time + duration as usize]
                                    + (requirements[index] as u32)
                                    > resource_limits[index] as u32
                                {
                                    start_time += 1;
                                    finished = false;
                                    break 'index_loop;
                                }
                            }
                        }

                        if finished {
                            // Put task resource requirements into resources vector
                            for index in 0..4 {
                                for duration in 0..self.durations[&self.job_to_nodes[job_id]] {
                                    resources[index][start_time + duration as usize] +=
                                        requirements[index] as u32;
                                }
                            }

                            break;
                        }
                    }
                }

                start_times.insert(*job_id, start_time);
            }
        }

        trace!("schedule: {schedule:?}");
        trace!("start_times: {start_times:?}");
        trace!("resources[0]: {:?}", resources[0]);
        trace!("resources[1]: {:?}", resources[1]);
        trace!("resources[2]: {:?}", resources[2]);
        trace!("resources[3]: {:?}", resources[3]);

        // Zip resource usage vectors together and filter out all unused space time slots
        let mut resources = resources;
        resources
            .remove(0)
            .into_iter()
            .zip(resources.remove(0))
            .zip(resources.remove(0))
            .zip(resources.remove(0))
            .map(|entry| (entry.0 .0 .0, entry.0 .0 .1, entry.0 .1, entry.1))
            .filter(|&element| element != (0, 0, 0, 0))
            .count()
    }
}
