use std::{fs::File, io::Write, path::PathBuf};

use clap::Parser;

mod generator;

/// Program to generate resource-constrained project scheduling problems
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Number of activites to generate
    #[clap(short, long, env, default_value_t = 10)]
    activity_count: u8,

    /// Pretty print generated problems in JSON
    #[clap(short, long, env)]
    pretty_json: bool,

    /// Target file for output
    #[clap(parse(from_os_str))]
    file: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args: Args = Args::parse();

    let activities = generator::generate_activities(args.activity_count);

    let writer = File::create(args.file)?;
    if args.pretty_json {
        serde_json::to_writer_pretty(&writer, &activities)?;
    } else {
        serde_json::to_writer(&writer, &activities)?;
    }

    let mut writer = writer;
    writer.flush()?;

    Ok(())
}
