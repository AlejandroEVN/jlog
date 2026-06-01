use std::{
    fmt::{Display, Formatter, Result},
    process,
    time::{SystemTime, UNIX_EPOCH},
};

use chrono::{DateTime, NaiveDateTime, Utc};
use clap::ValueEnum;
use colored::*;
use rusqlite::{
    ToSql,
    types::{FromSql, FromSqlResult, ToSqlOutput, Value, ValueRef},
};

#[derive(Debug, Clone, PartialEq, ValueEnum)]
pub enum JobStatus {
    Applied,
    InterviewStage,
    Declined,
    Offer,
    Accepted,
}

impl Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

impl JobStatus {
    fn from_str(s: &str) -> JobStatus {
        match s {
            "applied" => JobStatus::Applied,
            "interview" => JobStatus::InterviewStage,
            "declined" => JobStatus::Declined,
            "offer" => JobStatus::Offer,
            "accepted" => JobStatus::Accepted,
            _ => JobStatus::Applied,
        }
    }

    fn to_str(&self) -> &str {
        match self {
            JobStatus::Applied => "applied",
            JobStatus::InterviewStage => "interview",
            JobStatus::Declined => "declined",
            JobStatus::Offer => "offer",
            JobStatus::Accepted => "accepted",
        }
    }

    fn to_str_colored(&self) -> ColoredString {
        match self {
            JobStatus::Applied => "  Applied   ".blue().bold(),
            JobStatus::InterviewStage => "Interviewing".yellow().bold(),
            JobStatus::Declined => "  Declined  ".red().bold(),
            JobStatus::Offer => "  Offer     ".bright_green().bold(),
            JobStatus::Accepted => "  Accepted  ".green().bold(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct JobApplication {
    pub id: Option<i64>,
    pub title: String,
    pub company: String,
    pub location: String,
    pub url: String,
    pub applied_on: i64,
    pub updated_on: i64,
    pub status: JobStatus,
    pub next_interview_on: Option<i64>,
}

impl Display for JobApplication {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let next_interview_on = if let Some(next) = self.next_interview_on {
            JobApplication::format_timestamp(next)
        } else {
            "----------------".to_string()
        };

        write!(
            f,
            "[{}][{}] {:<55} @ {:<10} {}",
            next_interview_on,
            self.status.to_str_colored(),
            self.title.bold(),
            self.company,
            format!("({})", self.id.unwrap_or(0)).dimmed(),
        )?;

        Ok(())
    }
}

#[macro_export]
macro_rules! date_format_with_time {
    () => {
        format!(
            "{}{}{}",
            "%d/%m/%y".green(),
            " @ ".dimmed(),
            "%H:%M".green()
        )
    };
}

impl JobApplication {
    pub fn from_args(add_args: crate::args::AddArgs) -> Self {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards?")
            .as_millis();

        let next_interview_on_as_millis = add_args.next_interview_on.map(Self::timestamp_to_millis);

        JobApplication {
            id: None,
            title: add_args.title,
            company: add_args.company,
            location: add_args.location,
            url: add_args.url,
            applied_on: current_time as i64,
            updated_on: current_time as i64,
            status: add_args.state.unwrap_or(JobStatus::Applied),
            next_interview_on: next_interview_on_as_millis,
        }
    }

    pub fn timestamp_to_millis(timestamp: String) -> i64 {
        if let Ok(naive_datetime) = NaiveDateTime::parse_from_str(&timestamp, "%d/%m/%y@%H:%M") {
            let datetime_utc = naive_datetime.and_local_timezone(Utc).unwrap();
            datetime_utc.timestamp_millis()
        } else {
            eprintln!("Error parsing date. Date should be formatted as dd/mm/yy@HH:MM");
            process::exit(1);
        }
    }

    pub fn format_timestamp(ms: i64) -> String {
        DateTime::from_timestamp_millis(ms)
            .map(|dt| dt.format(date_format_with_time!().as_str()).to_string())
            .unwrap_or_else(|| "Invalid Date".to_string())
    }
}

impl ToSql for JobStatus {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(ToSqlOutput::Owned(Value::Text(self.to_str().to_string())))
    }
}

impl FromSql for JobStatus {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let state = value.as_str()?;

        Ok(JobStatus::from_str(state))
    }
}
