use clap::{Args, Parser, Subcommand};

use crate::job::JobStatus;

#[derive(Parser)]
#[command(name = "jlog")]
#[command(author = "Alejandro Noailles <vasconalex17@gmail.com>")]
#[command(version = "0.1")]
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
        /// Filter by specific job state
        #[arg(short, long, value_enum)]
        state: Option<JobStatus>,
    },

    /// Update the state of an existing job
    Status {
        /// The ID of the job in the database
        id: i64,
        /// The new state to transition to
        #[arg(value_enum)]
        status: JobStatus,
    },

    /// Update the state of an existing job
    Interview {
        /// The ID of the job in the database
        id: i64,

        /// The new interview date
        next_interview_on: String,
    },

    /// Delete a job application record
    Remove {
        /// The ID of the job to remove
        id: i64,
    },

    /// Print next upcoming interview
    Next,

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
    pub state: Option<JobStatus>,

    #[arg(short, long)]
    pub next_interview_on: Option<String>,
}
