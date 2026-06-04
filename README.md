# jlog

`jlog` is a lightweight, simple, terminal-based CLI tool designed to help track, organize, and manage their job applications without ever leaving the command line. Built with Rust, it uses a local SQLite database to store data, offers colored terminal output, and integrates smoothly with your system browser.

---

## Features

* **Fast Tracking:** Quickly log new job openings, companies, locations, and application links.
* **Status Lifecycle Management:** Move application status through `applied`, `interview`, `declined`, `offer`, and `accepted`.
* **Interview Scheduler:** Track upcoming interview dates and display what's next immediately.
* **Quick Links:** Open the original job posting URL directly in your default browser using the application ID.
* **Rich Terminal UI:** Colored, formatted outputs for high readability at a glance.

---

## Installation

### Method 1: Local Installation (macOS & Linux)

You can compile and install `jlog` directly to your local execution path using an automated shell script.

1. Clone this repository to your local machine.
2. Create an `install.sh` file using the script provided below.
3. Make the install script executable:
   ```bash
   chmod +x install.sh

4. Run the installer:
   ```bash
   ./install.sh
   ```



> **Note:** The script automatically detects if you are on Linux or macOS (Intel/Apple Silicon), compiles the binary in `--release` mode, and securely prompts for `sudo` only if administrative privileges are required to copy the file to your system's `$PATH`.

### Method 2: Download Pre-compiled Binaries

If you have set up continuous integration using the GitHub Actions workflow provided below, you can grab pre-built binaries without needing the Rust toolchain installed locally:

1. Navigate to the **Releases** section of your GitHub repository.
2. Download the compressed executable matched to your platform architecture:
* `jlog-linux-x86_64` (Linux)
* `jlog-mac-x86_64` (macOS Intel/Apple Silicon via Rosetta)


3. Move the binary into your system's global environment path variable.

## Usage Syntax

```text
Tracks job applications right from the terminal

Usage: jlog <COMMAND>

Commands:
  add        Add a new job application
  list       List all tracked job applications
  stats      Display job hunting stats
  interview  Add/update the interview date for a job application
  edit       Update the job application's metadata
  remove     Delete a job application record
  export     Export data into JSON or CSV file
  next       Print next upcoming interview
  open       Opens job application's URL in default browser
  help       Print this message or the help of the given subcommand(s)
```

---

## Date Format Reference

Whenever you input an interview timestamp, it must strictly follow this format:

> **`dd/mm/yy@HH:MM`** *(e.g., `15/10/26@14:30` for October 15, 2026 at 2:30 PM)*

---

## Command Examples

### 1. Add a New Job Application

Add a job by providing the core details. You can optionally include an initial status or a scheduled interview date.

```bash
jlog add --title "Software Engineer" --company "Acme Corp" --location "Remote" --url "https://jobs.acme.com/123"

```

*Include an initial state and interview:*

```bash
jlog add -t "Rust Developer" -c "Ferris Inc" -l "New York" -u "https://ferris.io/job" --status interview --next-interview-on "24/06/26@10:00"

```

## Job Lifecycle Status

The application recognizes the following valid status parameters (case-insensitive flag passing provided by `clap`):

| State Flag | Display Text |
| --- | --- |
| `applied` | Applied |
| `interview`| Interviewing |
| `declined` | Declined |
| `offer` | Offer |
| `accepted` | Accepted |

---

## License

This project is open-source. Feel free to tweak, fork, and adapt it to your personal career hunting workflows!
