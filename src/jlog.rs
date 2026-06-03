use std::error::Error;

use crate::{
    args::{AddArgs, EditArgs},
    db::{DB, JobQueryBuilder},
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

    pub(crate) fn list_jobs(
        &mut self,
        statuses: Option<Vec<JobStatus>>,
        prune: bool,
    ) -> Result<()> {
        let job_applications = self
            .db
            .get_job_applications(JobQueryBuilder::new().with_statuses(statuses).prune(prune))?;

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
}
