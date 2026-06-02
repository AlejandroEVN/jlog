use std::fmt::{self, Display, Formatter};

use chrono::{DateTime, NaiveDateTime, Utc};
use clap::ValueEnum;
use colored::{ColoredString, Colorize};
use rusqlite::{
    ToSql,
    types::{FromSql, FromSqlResult, ToSqlOutput, Value, ValueRef},
};

use crate::utils::Utils;

#[derive(Debug, Clone, PartialEq, Eq, ValueEnum)]
pub enum JobStatus {
    Applied,
    Interview,
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
    fn from_str(s: &str) -> Self {
        match s {
            "interview" => Self::Interview,
            "declined" => Self::Declined,
            "offer" => Self::Offer,
            "accepted" => Self::Accepted,
            _ => Self::Applied,
        }
    }

    const fn to_str(&self) -> &str {
        match self {
            Self::Applied => "applied",
            Self::Interview => "interview",
            Self::Declined => "declined",
            Self::Offer => "offer",
            Self::Accepted => "accepted",
        }
    }

    fn to_str_colored(&self) -> ColoredString {
        match self {
            Self::Applied => "  Applied   ".blue().bold(),
            Self::Interview => "Interviewing".yellow().bold(),
            Self::Declined => "  Declined  ".red().bold(),
            Self::Offer => "  Offer     ".bright_green().bold(),
            Self::Accepted => "  Accepted  ".green().bold(),
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
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let next_interview_on = self
            .next_interview_on
            .map_or_else(|| "----------------".to_string(), Self::format_timestamp);

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
    pub fn from_args(add_args: crate::args::AddArgs) -> Result<Self, String> {
        let current_time = Utils::get_current_time()?;

        let next_interview_on_as_millis = add_args
            .next_interview_on
            .map(|next| Self::timestamp_to_millis(&next))
            .transpose()?;

        Ok(Self {
            id: None,
            title: add_args.title,
            company: add_args.company,
            location: add_args.location,
            url: add_args.url,
            applied_on: current_time,
            updated_on: current_time,
            status: add_args.state.unwrap_or(JobStatus::Applied),
            next_interview_on: next_interview_on_as_millis,
        })
    }

    pub fn timestamp_to_millis(timestamp: &str) -> Result<i64, String> {
        let naive_datetime = NaiveDateTime::parse_from_str(timestamp, "%d/%m/%y@%H:%M")
            .map_err(|_| "Error parsing date. Date should be formatted as dd/mm/yy@HH:MM")?;

        naive_datetime
            .and_local_timezone(Utc)
            .single()
            .map(|dt| dt.timestamp_millis())
            .ok_or_else(|| "Invalid or ambiguous date/time specified".to_string())
    }

    pub fn format_timestamp(ms: i64) -> String {
        DateTime::from_timestamp_millis(ms).map_or_else(
            || "Invalid Date".to_string(),
            |dt| dt.format(date_format_with_time!().as_str()).to_string(),
        )
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

        Ok(Self::from_str(state))
    }
}
