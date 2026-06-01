use std::path::Path;

use crate::{args, job::JobApplication};

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
    _conn: Connection,
}

const TABLE_NAME: &str = "job_application";

impl DB {
    pub fn new(path: &Path) -> Self {
        let _conn = Connection::open(path.join("jlog.db")).expect("error: connecting to local db");

        _conn
            .execute(
                format!(
                    "CREATE TABLE IF NOT EXISTS {} (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                location TEXT NOT NULL,
                company TEXT NOT NULL,
                url TEXT NOT NULL,
                applied_on INTEGER NOT NULL,
                updated_on INTEGER NOT NULL,
                status TEXT NOT NULL,
                next_interview_on INTEGER
            );",
                    TABLE_NAME
                )
                .as_str(),
                (),
            )
            .expect("error: creating table");

        Self { _conn }
    }

    pub fn insert_job_application(&self, job: &JobApplication) -> rusqlite::Result<i64> {
        self._conn
            .execute(
                format!(
                    "INSERT INTO {} (
                title,
                location,
                company,
                url,
                applied_on,
                updated_on,
                status,
                next_interview_on
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8);",
                    TABLE_NAME
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
            .expect("error: inserting results");

        let last_inserted_id = self._conn.last_insert_rowid();

        Ok(last_inserted_id)
    }

    pub fn get_job_applications(&self) -> Vec<JobApplication> {
        let mut statement = self
            ._conn
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
                FROM {}
                ORDER BY applied_on DESC;",
                    TABLE_NAME
                )
                .as_str(),
            )
            .expect("error: getting job applications");

        statement
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
            .expect("error: executing GET query")
            .collect::<rusqlite::Result<Vec<JobApplication>>>()
            .expect("error: collecting results data")
    }

    pub fn delete_job_application(&self, id: i64) {
        self._conn
            .execute(
                format!("DELETE FROM {} WHERE id = ?1", TABLE_NAME).as_str(),
                params![id],
            )
            .expect("error: deleting job application");
    }

    pub(crate) fn update_job_application(&self, edit_args: args::EditArgs) {
        let mut assignments = Vec::new();
        let mut values: Vec<Box<dyn ToSql>> = Vec::new();

        push_optional_field!(assignments, values, edit_args.location, "location");
        push_optional_field!(assignments, values, edit_args.status, "status");
        push_optional_field!(assignments, values, edit_args.company, "company");
        push_optional_field!(assignments, values, edit_args.url, "url");
        push_optional_field!(assignments, values, edit_args.title, "title");

        if assignments.is_empty() {
            return;
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

        self._conn
            .execute(&query, params)
            .expect("error: updating job status");
    }

    pub(crate) fn update_next_interview_date(&self, id: i64, date_as_millis: Option<i64>) {
        self._conn
            .execute(
                format!(
                    "UPDATE {} SET next_interview_on = ?1, updated_on = ?2 WHERE id = ?3",
                    TABLE_NAME
                )
                .as_str(),
                params![date_as_millis, chrono::Utc::now().timestamp_millis(), id],
            )
            .expect("error: updating job next interview date");
    }
}
