use std::fs;

use anyhow::Result;
use log::trace;
use psp_lib_parser::parse_psp_lib;

use crate::Schedule;

pub fn schedule(schedule: Schedule) -> Result<()> {
    for input_file in schedule.input_files {
        let contents = fs::read_to_string(input_file)?;
        trace!("input file contents: {contents}");

        let psp = parse_psp_lib(contents.as_str())?;
        trace!("parsed psp: {psp:#?}");

        rcpsp::schedulers::simple::simple_schedule(psp, schedule.tabu_list_size);
    }

    Ok(())
}
