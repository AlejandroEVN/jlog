use std::io::{BufWriter, Stdout, Write, stdout};

use crate::job::JobApplication;

pub struct Printer {
    stdout: BufWriter<Stdout>,
}

impl Printer {
    pub(crate) fn new() -> Self {
        let stdout = stdout();
        let stdout_handle = BufWriter::new(stdout);

        Self {
            stdout: stdout_handle,
        }
    }

    fn flush(&mut self) {
        self.stdout.flush().unwrap();
    }

    pub(crate) fn print(&mut self, message: &str) {
        writeln!(self.stdout, "{message}").unwrap();

        self.flush();
    }

    pub(crate) fn job(&mut self, job_application: &JobApplication) {
        writeln!(self.stdout, "{job_application}").unwrap();

        self.flush();
    }

    pub(crate) fn job_added(&mut self, id: i64) {
        writeln!(self.stdout, "Job added <id:{id}>").unwrap();

        self.flush();
    }
}
