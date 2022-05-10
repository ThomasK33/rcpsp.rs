use anyhow::Result;
use log::{debug, info, trace};
use psp_lib_parser::{parse_psp_lib, structs::PspLibRequestDuration};
use std::{borrow::Cow, collections::HashMap, fs, path::PathBuf};

pub fn graph(input_path: PathBuf, output_path: PathBuf) -> Result<()> {
    let contents = fs::read_to_string(input_path)?;
    trace!("input file contents: {contents}");

    let psp = parse_psp_lib(contents.as_str())?;
    trace!("parsed psp: {psp:#?}");

    debug!(
        "Creating graph for file with basedata: {}",
        psp.file_with_basedata
    );

    let durations: HashMap<u8, PspLibRequestDuration> = psp
        .request_durations
        .into_iter()
        .map(|a| (a.job_number, a))
        .collect();

    let edges = Edges(
        psp.file_with_basedata.replace('.', "_"),
        durations,
        psp.precedence_relations
            .into_iter()
            .flat_map(|node| {
                node.successors
                    .into_iter()
                    .map(move |suc| (node.job_number, suc))
            })
            .collect(),
    );

    let mut output_file = std::fs::File::create(output_path.clone())?;
    dot::render(&edges, &mut output_file)?;

    info!("Wrote graphviz dot file to: {:?}", output_path);

    Ok(())
}

type Nd = u8;
type Ed = (u8, u8);
struct Edges(String, HashMap<u8, PspLibRequestDuration>, Vec<Ed>);

impl<'a> dot::Labeller<'a, Nd, Ed> for Edges {
    fn graph_id(&'a self) -> dot::Id<'a> {
        dot::Id::new(self.0.clone()).expect("Failed to get graph id")
    }

    fn node_id(&'a self, n: &Nd) -> dot::Id<'a> {
        let id = format!("N{}", *n);
        dot::Id::new(id).expect("Failed to label graph node")
    }

    fn node_label(&'a self, n: &Nd) -> dot::LabelText<'a> {
        let duration = self.1.get(n).map(|dur| dur.duration).unwrap_or_default();
        dot::LabelText::label(format!("{} ({})", *n, duration))
    }
}

impl<'a> dot::GraphWalk<'a, Nd, Ed> for Edges {
    fn nodes(&self) -> dot::Nodes<'a, Nd> {
        let v = &self.2;
        let mut nodes = Vec::with_capacity(v.len());
        for &(s, t) in v {
            nodes.push(s);
            nodes.push(t);
        }
        nodes.sort_unstable();
        nodes.dedup();
        Cow::Owned(nodes)
    }

    fn edges(&'a self) -> dot::Edges<'a, Ed> {
        let edges = &self.2;
        Cow::Borrowed(&edges[..])
    }

    fn source(&self, e: &Ed) -> Nd {
        e.0
    }

    fn target(&self, e: &Ed) -> Nd {
        e.1
    }
}
