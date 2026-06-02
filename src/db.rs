use std::{io::Error, path::Path};

use crate::{
    args,
    job::{JobApplication, JobStatus},
};

use rusqlite::{Connection, ToSql, params};

macro_rules! push_optional_field {
    ($assignments:ident, $values:expr, $field:expr, $column_name:expr) => {
        if let Some(val) = $field {
            $assignments.push(concat!($column_name, " = ?"));
            $values.push(Box::new(val) as Box<dyn rusqlite::ToSql>);
        }
    };
}

pub struct DB {
    conn: Connection,
}

#[derive(Debug, Clone, Default)]
pub struct JobQueryBuilder {
    id: Option<i64>,
    statuses: Option<Vec<JobStatus>>,
    prune: bool,
}

impl JobQueryBuilder {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) const fn with_id(mut self, id: i64) -> Self {
        self.id = Some(id);
        self
    }

    pub(crate) fn with_statuses(mut self, statuses: Option<Vec<JobStatus>>) -> Self {
        self.statuses = statuses;
        self
    }

    pub(crate) const fn prune(mut self, prune: bool) -> Self {
        self.prune = prune;
        self
    }

    fn into_where_clause(self) -> String {
        let mut where_clause = "TRUE".to_string();

        if let Some(statuses) = self.statuses {
            let as_string = statuses
                .iter()
                .map(|js| format!("\"{js}\""))
                .collect::<Vec<String>>()
                .join(",");

            let prune = if self.prune { "NOT IN" } else { "IN" };

            let where_status = format!("AND WHERE status {prune} ({as_string})");
            where_clause.push_str(&where_status);
        }

        if let Some(id) = self.id {
            let where_id = format!("AND WHERE id = {id}");
            where_clause.push_str(&where_id);
        }

        where_clause
    }
}

const TABLE_NAME: &str = "job_application";

impl DB {
    pub fn new(path: &Path) -> Result<Self, Error> {
        let conn = Connection::open(path.join("jlog.db"))
            .map_err(|_| Error::other("Error: Connecting to local db"))?;

        conn.execute(
            format!(
                "CREATE TABLE IF NOT EXISTS {TABLE_NAME} (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                location TEXT NOT NULL,
                company TEXT NOT NULL,
                url TEXT NOT NULL,
                applied_on INTEGER NOT NULL,
                updated_on INTEGER NOT NULL,
                status TEXT NOT NULL,
                next_interview_on INTEGER
            );"
            )
            .as_str(),
            (),
        )
        .map_err(|_| Error::other("Error: Creating table"))?;

        Ok(Self { conn })
    }

    pub(crate) fn get_one(&self, id: i64) -> Result<JobApplication, String> {
        self.get_job_applications(JobQueryBuilder::new().with_id(id))?
            .into_iter()
            .next()
            .ok_or_else(|| "Job application with id:{id} not found".to_string())
    }

    pub fn insert_job_application(&self, job: &JobApplication) -> Result<i64, String> {
        self.conn
            .execute(
                format!(
                    "INSERT INTO {TABLE_NAME} (
                title,
                location,
                company,
                url,
                applied_on,
                updated_on,
                status,
                next_interview_on
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8);"
                )
                .as_str(),
                params![
                    &job.title,
                    &job.location,
                    &job.company,
                    &job.url,
                    &job.applied_on,
                    &job.updated_on,
                    &job.status,
                    &job.next_interview_on,
                ],
            )
            .map_err(|_| "Error: Inserting results".to_string())?;

        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_job_applications(
        &self,
        query_builder: JobQueryBuilder,
    ) -> Result<Vec<JobApplication>, String> {
        let where_clause = query_builder.into_where_clause();

        let mut statement = self
            .conn
            .prepare(
                format!(
                    "
                SELECT 
                    id,
                    title,
                    location,
                    company,
                    url,
                    applied_on,
                    updated_on,
                    status,
                    next_interview_on
                FROM {TABLE_NAME}
                {where_clause}
                ORDER BY applied_on DESC;"
                )
                .as_str(),
            )
            .map_err(|_| "Error fetching job applications")?;

        let results = statement
            .query_map([], |row| {
                Ok(JobApplication {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    location: row.get(2)?,
                    company: row.get(3)?,
                    url: row.get(4)?,
                    applied_on: row.get(5)?,
                    updated_on: row.get(6)?,
                    status: row.get(7)?,
                    next_interview_on: row.get(8)?,
                })
            })
            .map_err(|_| "Error: parsing query results")?
            .collect::<rusqlite::Result<Vec<JobApplication>>>()
            .map_err(|_| "Error: collecting results data")?;

        Ok(results)
    }

    pub fn delete_job_application(&self, id: i64) -> Result<JobApplication, String> {
        let to_delete = self.get_one(id)?;

        self.conn
            .execute(
                format!("DELETE FROM {TABLE_NAME} WHERE id = ?1").as_str(),
                params![to_delete.id],
            )
            .map_err(|_| "Error: Deleting job application")?;

        Ok(to_delete)
    }

    pub(crate) fn update_job_application(
        &self,
        edit_args: args::EditArgs,
    ) -> Result<JobApplication, String> {
        let mut assignments = Vec::new();
        let mut values: Vec<Box<dyn ToSql>> = Vec::new();

        push_optional_field!(assignments, values, edit_args.location, "location");
        push_optional_field!(assignments, values, edit_args.status, "status");
        push_optional_field!(assignments, values, edit_args.company, "company");
        push_optional_field!(assignments, values, edit_args.url, "url");
        push_optional_field!(assignments, values, edit_args.title, "title");

        if assignments.is_empty() {
            return Err("Trying to update job application without passing any args".to_string());
        }

        assignments.push("updated_on = ?");
        values.push(Box::new(chrono::Utc::now().timestamp_millis()));

        values.push(Box::new(edit_args.id));

        let query = format!(
            "UPDATE {} SET {} WHERE id = ?",
            TABLE_NAME,
            assignments.join(", ")
        );

        let params = rusqlite::params_from_iter(values);

        self.conn
            .execute(&query, params)
            .map_err(|_| "Error: Updating job with id:{edit_args.id}")?;

        self.get_one(edit_args.id)
    }

    pub(crate) fn update_next_interview_date(
        &self,
        id: i64,
        date_as_millis: Option<i64>,
    ) -> Result<JobApplication, String> {
        self.conn
            .execute(
                format!(
                    "UPDATE {TABLE_NAME} SET next_interview_on = ?1, updated_on = ?2 WHERE id = ?3"
                )
                .as_str(),
                params![date_as_millis, chrono::Utc::now().timestamp_millis(), id],
            )
            .map_err(|_| "error: updating job next interview date".to_string())?;

        self.get_one(id)
    }
}
