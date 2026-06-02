#![warn(clippy::pedantic, clippy::nursery)]
mod args;
mod db;
mod job;

use clap::Parser;
use directories::ProjectDirs;
use std::{
    fs,
    io::{self, Write, stdout},
    time::{SystemTime, UNIX_EPOCH},
};

use args::Cli;
use db::DB;

use crate::{
    args::{AddArgs, EditArgs},
    db::JobQueryBuilder,
    job::JobApplication,
};

fn main() {
    let directory = init();

    let db = DB::new(directory.config_dir());

    let cli_args = Cli::parse();

    match cli_args.command {
        args::Commands::Add(add_args) => JLog::add_job(&db, add_args),
        args::Commands::List { state, prune } => JLog::list_jobs(&db, state, prune),
        args::Commands::Remove { id } => JLog::remove_job(&db, id),
        args::Commands::Next { days } => JLog::find_next_interview(&db, days),
        args::Commands::Interview {
            id,
            next_interview_on,
            clear,
        } => JLog::add_next_interview_date(&db, id, next_interview_on, clear),
        args::Commands::Open { id } => JLog::open_job_url(&db, id),
        args::Commands::Edit(edit_args) => JLog::update_job_application(&db, edit_args),
    }
}

struct JLog {}

impl JLog {
    fn list_jobs(db: &DB, statuses: Option<Vec<job::JobStatus>>, prune: bool) {
        let stdout = stdout();
        let mut handle = io::BufWriter::new(stdout);

        let job_applications =
            db.get_job_applications(JobQueryBuilder::new().with_statuses(statuses).prune(prune));

        for app in job_applications {
            writeln!(handle, "{}", &app).unwrap();
        }
    }

    fn find_next_interview(db: &DB, days: usize) {
        let stdout = stdout();
        let mut handle = io::BufWriter::new(stdout);
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards?")
            .as_millis();

        let mut job_applications = db.get_job_applications(JobQueryBuilder::default());

        job_applications.sort_by_key(|ja| (ja.next_interview_on.is_none(), ja.next_interview_on));

        job_applications
            .iter()
            .filter(|ja| ja.next_interview_on.unwrap_or(0) > i64::try_from(current_time).unwrap())
            .take(days)
            .for_each(|ja| {
                if ja.next_interview_on.is_some() {
                    writeln!(handle, "{}", &ja).unwrap();
                }
            });
    }

    fn add_job(db: &DB, add_args: AddArgs) {
        let stdout = stdout();
        let mut handle = io::BufWriter::new(stdout);

        let job_application = JobApplication::from_args(add_args);

        let id = db.insert_job_application(&job_application);

        writeln!(handle, "Job added <id:{id}>").unwrap();
    }

    fn remove_job(db: &DB, id: i64) {
        db.delete_job_application(id);
    }

    fn add_next_interview_date(db: &DB, id: i64, next_interview_on: Option<String>, clear: bool) {
        let date_as_millis = match (clear, next_interview_on) {
            (true, _) | (false, None) => None,
            (false, Some(date)) => Some(JobApplication::timestamp_to_millis(&date)),
        };

        db.update_next_interview_date(id, date_as_millis);
    }

    fn open_job_url(db: &DB, id: i64) {
        let stdout = stdout();
        let mut handle = io::BufWriter::new(stdout);

        let job_applications = db.get_job_applications(JobQueryBuilder::default());

        if let Some(ja) = job_applications.iter().find(|ja| ja.id == Some(id)) {
            if open::that(&ja.url).is_ok() {
                writeln!(handle, "Opening URL {}", &ja.url).unwrap();
            } else {
                eprintln!("Failed to open URL: {}", &ja.url);
            }
        } else {
            eprintln!("Job application with ID {id} not found");
        }
    }

    fn update_job_application(db: &DB, edit_args: EditArgs) {
        db.update_job_application(edit_args);
    }
}

fn init() -> ProjectDirs {
    let project_dirs =
        ProjectDirs::from("", "", "jlog").expect("error: could not determine project directories");

    fs::create_dir_all(project_dirs.data_dir()).expect("error: creating .local data folder");

    project_dirs
}
