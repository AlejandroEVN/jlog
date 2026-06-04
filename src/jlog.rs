use core::fmt;
use std::{
    error::Error,
    ffi::OsStr,
    fmt::{Display, Formatter},
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use colored::{ColoredString, Colorize};

use crate::{
    args::{self, AddArgs, EditArgs, FileFormat},
    db::{DB, JobQueryBuilder},
    jlog,
    job::{JobApplication, JobStatus},
    printer::Printer,
    utils::Utils,
};

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub struct JLog<'a> {
    db: &'a DB,
    printer: Printer,
}

impl<'a> JLog<'a> {
    pub(crate) const fn new(db: &'a DB, printer: Printer) -> Self {
        Self { db, printer }
    }

    pub(crate) fn list_jobs(&mut self, query_builder: JobQueryBuilder) -> Result<()> {
        let job_applications = self.db.get_job_applications(query_builder)?;

        for app in job_applications {
            self.printer.job(&app);
        }

        Ok(())
    }

    pub(crate) fn find_next_interview(&mut self, days: usize) -> Result<()> {
        let current_time = Utils::get_current_time()?;

        let mut job_applications = self.db.get_job_applications(JobQueryBuilder::default())?;

        job_applications.sort_by_key(|ja| (ja.next_interview_on.is_none(), ja.next_interview_on));

        job_applications
            .iter()
            .filter(|ja| ja.next_interview_on.unwrap_or(0) > current_time)
            .take(days)
            .for_each(|ja| {
                if ja.next_interview_on.is_some() {
                    self.printer.job(ja);
                }
            });

        Ok(())
    }

    pub(crate) fn add_job(&mut self, add_args: AddArgs) -> Result<()> {
        let job_application = JobApplication::from_args(add_args)?;

        let id = self.db.insert_job_application(&job_application)?;

        self.printer.job_added(id);

        Ok(())
    }

    pub(crate) fn remove_job(&mut self, id: i64) -> Result<()> {
        let deleted = self.db.delete_job_application(id)?;

        self.printer.job(&deleted);

        Ok(())
    }

    pub(crate) fn add_next_interview_date(
        &mut self,
        id: i64,
        next_interview_on: Option<String>,
        clear: bool,
    ) -> Result<()> {
        let date_as_millis = match (clear, next_interview_on) {
            (true, _) | (false, None) => None,
            (false, Some(date)) => Some(JobApplication::timestamp_to_millis(&date)?),
        };

        let updated = self.db.update_next_interview_date(id, date_as_millis)?;

        self.printer.job(&updated);

        Ok(())
    }

    pub(crate) fn open_job_url(&mut self, id: i64) -> Result<()> {
        let job_application = self.db.get_one(id)?;

        open::that(&job_application.url)?;

        self.printer.print("Opening URL {&ja.url}");

        Ok(())
    }

    pub(crate) fn update_job_application(&mut self, edit_args: EditArgs) -> Result<()> {
        let updated = self.db.update_job_application(edit_args)?;

        self.printer.job(&updated);

        Ok(())
    }

    #[allow(clippy::cast_precision_loss)]
    pub(crate) fn display_stats(&mut self) -> jlog::Result<()> {
        let job_applications = self.db.get_job_applications(JobQueryBuilder::default())?;

        let mut stats = Stats {
            total: job_applications.len(),
            ..Default::default()
        };

        if stats.total == 0 {
            self.printer.print(&stats.to_string());
            return Ok(());
        }

        for ja in &job_applications {
            match ja.status {
                crate::job::JobStatus::Applied => stats.applied += 1,
                crate::job::JobStatus::Interview => stats.interview += 1,
                crate::job::JobStatus::Declined => stats.declined += 1,
                crate::job::JobStatus::Offer => stats.offer += 1,
                crate::job::JobStatus::Accepted => stats.accepted += 1,
            }
        }

        let positive_outcomes = (stats.interview + stats.offer + stats.accepted) as f64;
        stats.conversion_rate = positive_outcomes / stats.total as f64;

        self.printer.print(&stats.to_string());

        Ok(())
    }

    pub(crate) fn export_data(
        &self,
        format: Option<args::FileFormat>,
        output: Option<String>,
    ) -> jlog::Result<()> {
        let file_format = format.map_or(args::FileFormat::Json, |f| f);
        let output_file = output.unwrap_or_else(|| file_format.output());

        let output_as_path = PathBuf::from_str(&output_file)?;

        Self::check_file_extension(&output_as_path, file_format)?;

        let job_applications = self.db.get_job_applications(JobQueryBuilder::default())?;

        let data = match file_format {
            args::FileFormat::Json => serde_json::to_string(&job_applications)?,
            args::FileFormat::Csv => Self::serialize_to_csv(&job_applications)?,
        };

        fs::write(output_file, data)?;

        Ok(())
    }

    fn check_file_extension(output_as_path: &Path, file_format: FileFormat) -> Result<()> {
        if output_as_path.extension() != Some(OsStr::new(file_format.extension())) {
            let expected_path = output_as_path
                .extension()
                .map_or_else(|| "none".to_string(), |e| e.to_string_lossy().into_owned());

            return Err(Box::<dyn Error>::from(format!(
                "File extension \"{}\" doesn't match format {:?}",
                expected_path,
                file_format.extension(),
            )));
        }

        Ok(())
    }

    fn serialize_to_csv(job_applications: &Vec<JobApplication>) -> Result<String> {
        let mut writer = csv::Writer::from_writer(vec![]);

        for ja in job_applications {
            writer.serialize(ja)?;
        }

        let inner_buffer = writer.into_inner()?;
        let csv_string = String::from_utf8(inner_buffer)?;

        Ok(csv_string)
    }
}

pub enum StatMetric {
    Total,
    Applied,
    Interview,
    Declined,
    Offer,
    Accepted,
    ConversionRate,
}

impl StatMetric {
    const fn raw_title(&self) -> &'static str {
        match self {
            Self::Total => "Total Applications",
            Self::Applied => "Applied",
            Self::Interview => "Interview",
            Self::Declined => "Declined",
            Self::Offer => "Offers",
            Self::Accepted => "Accepted",
            Self::ConversionRate => "Conversion Rate",
        }
    }

    pub(crate) fn to_str_colored(&self) -> ColoredString {
        match self {
            Self::Applied => self.raw_title().blue().bold(),
            Self::Interview => self.raw_title().yellow().bold(),
            Self::Declined => self.raw_title().red().bold(),
            Self::Offer => self.raw_title().purple().bold(),
            Self::Accepted => self.raw_title().green().bold(),
            Self::Total | Self::ConversionRate => self.raw_title().bold(),
        }
    }
}

#[derive(Debug, Default)]
struct Stats {
    total: usize,
    applied: usize,
    interview: usize,
    declined: usize,
    offer: usize,
    accepted: usize,
    conversion_rate: f64,
}

impl Stats {
    const fn calculate_percentage(&self, metric: &StatMetric) -> f64 {
        if self.total == 0 {
            return 0.0;
        }

        #[allow(clippy::cast_precision_loss)]
        match metric {
            StatMetric::Applied => (self.applied as f64 / self.total as f64) * 100.0,
            StatMetric::Interview => (self.interview as f64 / self.total as f64) * 100.0,
            StatMetric::Declined => (self.declined as f64 / self.total as f64) * 100.0,
            StatMetric::Offer => (self.offer as f64 / self.total as f64) * 100.0,
            StatMetric::Accepted => (self.accepted as f64 / self.total as f64) * 100.0,
            StatMetric::Total => {
                if self.total > 0 {
                    100.0
                } else {
                    0.0
                }
            }
            StatMetric::ConversionRate => self.conversion_rate,
        }
    }

    const fn count(&self, metric: &StatMetric) -> usize {
        match metric {
            StatMetric::Total => self.total,
            StatMetric::Applied => self.applied,
            StatMetric::Interview => self.interview,
            StatMetric::Declined => self.declined,
            StatMetric::Offer => self.offer,
            StatMetric::Accepted => self.accepted,
            StatMetric::ConversionRate => 0,
        }
    }
}

impl From<&JobStatus> for StatMetric {
    fn from(status: &JobStatus) -> Self {
        match status {
            JobStatus::Applied => Self::Applied,
            JobStatus::Interview => Self::Interview,
            JobStatus::Declined => Self::Declined,
            JobStatus::Offer => Self::Offer,
            JobStatus::Accepted => Self::Accepted,
        }
    }
}

impl Display for Stats {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let title_width = 20;
        let count_width = 4;
        let precision = 2;

        let header = "----- JLOG JOB HUNT STATISTICS -----\n";

        writeln!(f, "{header}")?;
        writeln!(
            f,
            "{:<title_width$} {:>count_width$}",
            StatMetric::Total.to_str_colored(),
            self.total
        )?;

        let core_stats = [
            StatMetric::Applied,
            StatMetric::Interview,
            StatMetric::Declined,
            StatMetric::Offer,
            StatMetric::Accepted,
        ];

        for stat in core_stats {
            writeln!(
                f,
                "{:title_width$} {:>count_width$} ({:.precision$}%)",
                stat.to_str_colored(),
                self.count(&stat),
                self.calculate_percentage(&stat)
            )?;
        }
        writeln!(
            f,
            "\n{:title_width$} {:>7.precision$}%",
            StatMetric::ConversionRate.to_str_colored(),
            self.calculate_percentage(&StatMetric::ConversionRate),
        )?;

        Ok(())
    }
}
