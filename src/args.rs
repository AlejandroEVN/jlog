use clap::{Args, Parser, Subcommand};

use crate::job::JobStatus;

#[derive(Parser)]
#[command(name = "jlog")]
#[command(author = "Alejandro Noailles <vasconalex17@gmail.com>")]
#[command(version = "1.0.2")]
#[command(about = "Tracks job applications right from the terminal", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a new job application
    Add(AddArgs),

    /// List all tracked job applications
    List {
        /// Filter by specific job status
        #[clap(value_delimiter = ' ')]
        status: Option<Vec<JobStatus>>,

        /// Filter by company name
        #[clap(short, long)]
        company: Option<String>,

        /// Filter by location
        #[clap(short, long)]
        location: Option<String>,

        #[arg(long)]
        prune: bool,
    },

    /// Edit metadata of job application entry
    Edit(EditArgs),

    /// Update the status of an existing job
    Interview {
        /// The ID of the job in the database
        id: i64,

        /// The new interview date
        #[arg(required_unless_present = "clear")]
        next_interview_on: Option<String>,

        /// Clear the scheduled interview date entirely
        #[arg(long, short, conflicts_with = "next_interview_on")]
        clear: bool,
    },

    /// Delete a job application record
    Remove {
        /// The ID of the job to remove
        id: i64,
    },

    /// Print next upcoming interview
    Next {
        /// Number of days ahead to return
        #[arg(default_value_t = 1)]
        days: usize,
    },

    /// See your job hunting stats
    Stats,

    /// Opens job application's URL in default browser
    Open {
        /// The ID of the job in the database
        id: i64,
    },
}

#[derive(Args)]
pub struct AddArgs {
    #[arg(short, long)]
    pub location: String,

    #[arg(short, long)]
    pub title: String,

    #[arg(short, long)]
    pub company: String,

    #[arg(short, long)]
    pub url: String,

    #[arg(short, long)]
    pub status: Option<JobStatus>,

    #[arg(short, long)]
    pub next_interview_on: Option<String>,
}

#[derive(Args)]
pub struct EditArgs {
    pub id: i64,

    #[arg(short, long)]
    pub location: Option<String>,

    #[arg(short, long)]
    pub title: Option<String>,

    #[arg(short, long)]
    pub company: Option<String>,

    #[arg(short, long)]
    pub url: Option<String>,

    #[arg(short, long, value_enum)]
    pub status: Option<JobStatus>,
}
