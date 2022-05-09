use anyhow::Result;
use log::trace;
use psp_lib_parser::parse_psp_lib;
use std::{fs, path::PathBuf};

pub fn graph(input_path: PathBuf, output_path: PathBuf) -> Result<()> {
    let contents = fs::read_to_string(input_path)?;
    trace!("input file contents: {contents}");

    let psp = parse_psp_lib(contents.as_str())?;
    trace!("parsed psp: {psp:#?}");

    Ok(())
}
