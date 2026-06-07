use std::path::Path;

use crate::{
    args, jlog,
    job::{JobApplication, JobStatus},
};

use rusqlite::{Connection, ToSql, params};

macro_rules! push_optional_field {
    ($assignments:ident, $values:expr, $field:expr, $column_name:expr, $should_clear:expr) => {
        if $should_clear {
            $assignments.push(concat!($column_name, " = NULL"));
        } else if let Some(val) = $field {
            $assignments.push(concat!($column_name, " = ?"));
            $values.push(Box::new(val));
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
    company: Option<String>,
    prune: bool,
    location: Option<String>,
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

    pub(crate) fn with_company_name(mut self, company: Option<String>) -> Self {
        self.company = company;
        self
    }

    pub(crate) fn with_location(mut self, location: Option<String>) -> Self {
        self.location = location;
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
            let prune = if self.prune { "!=" } else { "=" };

            let where_id = format!("AND WHERE id {prune} {id}");
            where_clause.push_str(&where_id);
        }

        if let Some(company) = self.company {
            let prune = if self.prune { "NOT LIKE" } else { "LIKE" };

            let where_company = format!("AND WHERE LOWER(company) {prune} LOWER('%{company}%')");
            where_clause.push_str(&where_company);
        }

        if let Some(location) = self.location {
            let prune = if self.prune { "NOT LIKE" } else { "LIKE" };

            let where_location = format!("AND WHERE LOWER(location) {prune} LOWER('%{location}%')");
            where_clause.push_str(&where_location);
        }

        where_clause
    }
}

const TABLE_NAME: &str = "job_application";

impl DB {
    pub fn new(path: &Path) -> jlog::Result<Self> {
        let conn = Connection::open(path.join("jlog.db"))?;

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
        )?;

        Ok(Self { conn })
    }

    pub(crate) fn get_one(&self, id: i64) -> jlog::Result<JobApplication> {
        self.get_job_applications(JobQueryBuilder::new().with_id(id))?
            .into_iter()
            .next()
            .ok_or_else(|| {
                Box::<dyn std::error::Error>::from(format!(
                    "Job application with id:{id} not found"
                ))
            })
    }

    pub fn insert_job_application(&self, job: &JobApplication) -> jlog::Result<i64> {
        self.conn.execute(
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
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_job_applications(
        &self,
        query_builder: JobQueryBuilder,
    ) -> jlog::Result<Vec<JobApplication>> {
        let where_clause = query_builder.into_where_clause();

        let mut statement = self.conn.prepare(
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
        )?;

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
            })?
            .collect::<rusqlite::Result<Vec<JobApplication>>>()?;

        Ok(results)
    }

    pub fn delete_job_application(&self, id: i64) -> jlog::Result<JobApplication> {
        let to_delete = self.get_one(id)?;

        self.conn.execute(
            format!("DELETE FROM {TABLE_NAME} WHERE id = ?1").as_str(),
            params![to_delete.id],
        )?;

        Ok(to_delete)
    }

    pub(crate) fn update_job_application(
        &self,
        edit_args: args::EditArgs,
    ) -> jlog::Result<JobApplication> {
        let mut assignments = Vec::new();
        let mut values: Vec<Box<dyn ToSql>> = Vec::new();

        push_optional_field!(assignments, values, edit_args.location, "location", false);
        push_optional_field!(assignments, values, edit_args.status, "status", false);
        push_optional_field!(assignments, values, edit_args.company, "company", false);
        push_optional_field!(assignments, values, edit_args.url, "url", false);
        push_optional_field!(assignments, values, edit_args.title, "title", false);
        if edit_args.next_interview_on.is_some() || edit_args.clear_interview {
            let date_as_millis = match edit_args.next_interview_on {
                Some(date_str) => Some(JobApplication::timestamp_to_millis(&date_str)?),
                None => Some(0),
            };

            push_optional_field!(
                assignments,
                values,
                date_as_millis,
                "next_interview_on",
                edit_args.clear_interview
            );
        }

        if assignments.is_empty() {
            return Err(Box::<dyn std::error::Error>::from(
                "Trying to update job application without passing any args".to_string(),
            ));
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

        self.conn.execute(&query, params)?;

        self.get_one(edit_args.id)
    }
}
