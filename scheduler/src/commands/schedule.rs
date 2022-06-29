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

        rcpsp::schedulers::simple::simple_schedule(
            psp,
            rcpsp::schedulers::simple::SimpleScheduleOptions {
                number_of_iterations: schedule.number_of_iterations,
                max_iter_since_best: schedule.max_iter_since_best,
                tabu_list_size: schedule.tabu_list_size,
                swap_range: schedule.swap_range,
            },
        );
    }

    Ok(())
}
