use std::collections::{HashMap, HashSet};

use log::trace;
use petgraph::algo;
use petgraph::visit::NodeIndexable;
use psp_lib_parser::structs::PspLibProblem;

use crate::sources_load::{self, EvaluationAlgorithm, SourcesLoad};

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

    // requests: HashMap<u8, (u8, u8, u8, u8)>,
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
            psp,
            job_to_nodes,
            node_to_jobs,
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
        // Reduced neighborhood to initial solution depends on the neighborhood size
        // parameter (swap range) and is an upper bound for move generation

        let mut windows = schedule.windows(swap_range).peekable();

        let mut moves = vec![];

        while let Some(window) = windows.next() {
            // Map each window to moves
            if let Some(first) = window.first() {
                let mut neighbors: Vec<(u8, u8)> = window
                    .iter()
                    .skip(1)
                    .map(|neighbor| (*first.min(neighbor), *neighbor.max(first)))
                    .collect();

                moves.append(&mut neighbors);
            }

            // Special case for last window
            if windows.peek().is_none() {
                // Create smaller sub-windows with less elements than swap_range yet the
                // possibility to yield valid moves

                let mut last_window = window.to_vec();
                last_window.append(&mut vec![0; swap_range as usize]);

                for window in last_window.windows(swap_range) {
                    if let Some(first) = window.first() {
                        let mut neighbors: Vec<(u8, u8)> = window
                            .iter()
                            .skip(1)
                            .filter(|neighbor| **neighbor != 0)
                            .map(|neighbor| (*first.min(neighbor), *neighbor.max(first)))
                            .collect();

                        moves.append(&mut neighbors);
                    }
                }
            }
        }

        // Filter out infeasible moves, i.e. moves that violate a precedence relation
        moves
            .into_iter()
            .filter(|(u, v)| {
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
                            algo::all_simple_paths::<Vec<_>, _>(
                                &self.graph,
                                self.job_to_nodes[node],
                                self.job_to_nodes[nodes_between.last().unwrap()],
                                0,
                                None,
                            )
                            .count()
                                == 0
                                && algo::all_simple_paths::<Vec<_>, _>(
                                    &self.graph,
                                    self.job_to_nodes[nodes_between.first().unwrap()],
                                    self.job_to_nodes[node],
                                    0,
                                    None,
                                )
                                .count()
                                    == 0
                        });
                    }
                }

                false
            })
            .collect()
    }

    pub fn evaluate_order(
        &self,
        forward_evaluation: bool,
        algorithm: EvaluationAlgorithm,
        solution: &[usize],
    ) -> usize {
        let number_of_resources = self.psp.resources.renewable;
        let capacity_of_resources = vec![
            self.psp.resource_availabilities.r1 as usize,
            self.psp.resource_availabilities.r2 as usize,
            self.psp.resource_availabilities.r3 as usize,
            self.psp.resource_availabilities.r4 as usize,
        ];

        let _sources_load: Box<dyn SourcesLoad> = match algorithm {
            EvaluationAlgorithm::CapacityResolution => Box::new(
                sources_load::CapacityResolution::new(number_of_resources, capacity_of_resources),
            ),
            EvaluationAlgorithm::TimeResolution => Box::new(sources_load::TimeResolution::new(
                number_of_resources,
                capacity_of_resources,
                (&self.psp.request_durations)
                    .iter()
                    .map(|duration| duration.duration as usize)
                    .sum(),
            )),
        };

        let schedule_length: usize = 0;

        for i in 0..self.psp.jobs {
            let _start: usize = 0;

            if let Some(&_activity_id) = solution.get(if forward_evaluation {
                i
            } else {
                self.psp.jobs - i - 1
            }) {
                // TODO: Implement order evaluation
                todo!("Implement order evaluation")
            }
        }

        schedule_length
    }

    pub fn compute_execution_time(&self, schedule: &[u8]) -> usize {
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
                        .map(|&duration| duration)
                        .unwrap_or_else(|| 0) as usize;

                    *start_times.get(&job_number).unwrap_or_else(|| &0) + duration
                })
                .max();

            if let Some(start_time) = start_time {
                let mut start_time = start_time;

                // Once the earliest start time has been determined, try fitting the task into the resources vector
                if let Some(requirements) = self.requests.get(&job_id) {
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
                        'index_loop: for index in 0..4 {
                            for duration in 0..self.durations[&self.job_to_nodes[job_id]] {
                                if resources[index][start_time + duration as usize]
                                    + (requirements[index] as u32)
                                    > resource_limits[index] as u32
                                {
                                    start_time += 1;
                                    break 'index_loop;
                                }
                            }
                        }

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
