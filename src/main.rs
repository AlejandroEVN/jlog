#![warn(clippy::pedantic, clippy::nursery)]
mod args;
mod db;
mod jlog;
mod job;
mod printer;
mod utils;

use clap::Parser;
use directories::ProjectDirs;
use std::{fs, process};

use args::Cli;
use db::DB;
use jlog::JLog;
use printer::Printer;

use crate::db::JobQueryBuilder;

fn main() -> jlog::Result<()> {
    let directory = init()?;

    let db = DB::new(directory.config_dir())?;

    let printer = Printer::new();
    let mut jlog = JLog::new(&db, printer);

    let cli_args = Cli::parse();

    let result = match cli_args.command {
        args::Commands::Add(add_args) => jlog.add_job(add_args),
        args::Commands::List {
            status,
            company,
            location,
            prune,
        } => jlog.list_jobs(
            JobQueryBuilder::new()
                .with_company_name(company)
                .with_statuses(status)
                .with_location(location)
                .prune(prune),
        ),
        args::Commands::Remove { id } => jlog.remove_job(id),
        args::Commands::Next { days } => jlog.find_next_interview(days),
        args::Commands::Open { id } => jlog.open_job_url(id),
        args::Commands::Edit(edit_args) => jlog.update_job_application(edit_args),
        args::Commands::Stats => jlog.display_stats(),
        args::Commands::Export { format, output } => jlog.export_data(format, output),
    };

    if let Err(err) = result {
        eprintln!("{err}");
        process::exit(1);
    }

    process::exit(0);
}

fn init() -> jlog::Result<ProjectDirs> {
    let dirs =
        ProjectDirs::from("", "", "jlog").ok_or("Failed to determine project directories")?;

    fs::create_dir_all(dirs.data_dir())?;

    Ok(dirs)
}
