use anyhow::Result;
use log::{debug, info, trace};
use psp_lib_parser::{parse_psp_lib, PspLibRequestDuration};
use std::{
    collections::{HashMap, HashSet},
    fs,
    io::Write,
    path::PathBuf,
};

pub fn graph(input_path: PathBuf, output_path: PathBuf) -> Result<()> {
    let contents = fs::read_to_string(input_path)?;
    trace!("input file contents: {contents}");

    let psp = parse_psp_lib(contents.as_str())?;
    trace!("parsed psp: {psp:#?}");

    debug!(
        "Creating graph for file with basedata: {}",
        psp.file_with_basedata
    );

    let output_file = std::fs::File::create(output_path.clone())?;
    write_dot_file(output_file, psp)?;

    info!("Wrote graphviz dot file to: {:?}", output_path);

    Ok(())
}

fn write_dot_file(mut file: fs::File, psp: psp_lib_parser::structs::PspLibProblem) -> Result<()> {
    let durations: HashMap<u8, PspLibRequestDuration> = psp
        .request_durations
        .into_iter()
        .map(|a| (a.job_number, a))
        .collect();

    let title = psp.file_with_basedata.replace('.', "_");

    file.write_all(b"digraph ")?;
    file.write_all(title.as_bytes())?;
    file.write_all(b" {\n")?;
    file.write_all(b"\trankdir=LR;\n")?;
    file.write_all(b"\tconcentrate=true;\n\n")?;

    let mut successor_map = HashMap::new();

    for (k, v) in durations {
        let duration = v.duration;
        file.write_all(format!("\t{k} [label=\"{k} ({duration})\"]\n").as_bytes())?;
    }

    file.write_all(b"\n")?;

    // Write nodes
    for node in psp.precedence_relations {
        if node.successor_count == 0 {
            continue;
        }

        file.write_all(b"\t")?;
        file.write_all(node.job_number.to_string().as_bytes())?;
        file.write_all(b" -> { ")?;
        file.write_all(
            node.successors
                .iter()
                .map(|idx| format!("{}", *idx,))
                .collect::<Vec<String>>()
                .join(" ")
                .as_bytes(),
        )?;
        file.write_all(b" };\n")?;

        successor_map.insert(node.job_number, node.successors);
    }

    file.write_all(b"\n")?;

    let mut prerequisite_map: HashMap<u8, Vec<u8>> = HashMap::new();

    for (job_number, successors) in successor_map.iter() {
        for successor in successors {
            if let Some(requirements) = prerequisite_map.get_mut(successor) {
                requirements.push(*job_number);
            } else {
                prerequisite_map.insert(*successor, vec![*job_number]);
            }
        }
    }

    // Calculate same execution ranks
    // ranks = dependency ranks
    let mut ranks: Vec<Vec<u8>> = vec![];
    {
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
    }

    for rank in ranks.into_iter() {
        file.write_all(b"\t{ rank=same; ")?;
        file.write_all(
            rank.into_iter()
                .map(|idx| format!("{}", idx))
                .collect::<Vec<String>>()
                .join(" ")
                .as_bytes(),
        )?;
        file.write_all(b" }\n")?;
    }

    // Close graph
    file.write_all(b"\n}\n")?;
    file.flush()?;

    Ok(())
}
