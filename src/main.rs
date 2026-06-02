#![warn(clippy::pedantic, clippy::nursery)]
mod args;
mod db;
mod jlog;
mod job;
mod printer;
mod utils;

use clap::Parser;
use directories::ProjectDirs;
use std::{fs, io::Error, process};

use args::Cli;
use db::DB;
use jlog::JLog;
use printer::Printer;

fn main() -> Result<(), Error> {
    let directory = init()?;

    let db = DB::new(directory.config_dir())?;

    let printer = Printer::new();
    let mut jlog = JLog::new(&db, printer);

    let cli_args = Cli::parse();

    match cli_args.command {
        args::Commands::Add(add_args) => jlog.add_job(add_args),
        args::Commands::List { state, prune } => jlog.list_jobs(state, prune),
        args::Commands::Remove { id } => jlog.remove_job(id),
        args::Commands::Next { days } => jlog.find_next_interview(days),
        args::Commands::Interview {
            id,
            next_interview_on,
            clear,
        } => jlog.add_next_interview_date(id, next_interview_on, clear),
        args::Commands::Open { id } => jlog.open_job_url(id),
        args::Commands::Edit(edit_args) => jlog.update_job_application(edit_args),
    }
    .map_err(Error::other)?;

    process::exit(0);
}

fn init() -> Result<ProjectDirs, Error> {
    let project_dirs = ProjectDirs::from("", "", "jlog");

    project_dirs
        .ok_or_else(|| {
            Error::new(
                std::io::ErrorKind::NotFound,
                "error: could not determine project directories",
            )
        })
        .and_then(|dirs| {
            fs::create_dir_all(dirs.data_dir())?;

            Ok(dirs)
        })
}
