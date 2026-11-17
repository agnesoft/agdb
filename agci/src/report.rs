use crate::CIError;
use agplatform::Env;
use agplatform::Fs;
use agplatform::OpenWriteOptions;
use agplatform::Platform;
use std::io::Write;
use std::time::Duration;
use std::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Status {
    Passed,
    Failed,
    Skipped,
    /// Passed before with the same fingerprint, not run.
    Proven,
}

/// What a row of the report is about.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Task<'a> {
    /// The setup of the project's kind (e.g. `pnpm install`), before its first check that runs.
    Setup,
    Check(&'a str),
}

impl std::fmt::Display for Task<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Task::Setup => "setup",
            Task::Check(name) => name,
        })
    }
}

#[derive(Debug)]
struct Entry {
    project: String,
    check: String,
    status: Status,
    duration: Duration,
}

/// Results of the checks. On GitHub Actions each check is a collapsible log group, a failure is an
/// error annotation and the results are added to the job summary.
pub(crate) struct Report {
    github: bool,
    entries: Vec<Entry>,
}

impl Report {
    pub(crate) fn new(platform: &impl Platform) -> Self {
        Self {
            github: platform
                .env()
                .var("GITHUB_ACTIONS")
                .is_some_and(|v| v == "true"),
            entries: Vec::new(),
        }
    }

    /// Returns whether the check passed.
    pub(crate) fn run(
        &mut self,
        project: &str,
        task: Task,
        run: impl FnOnce() -> Result<(), CIError>,
    ) -> bool {
        let title = format!("{project}: {task}");

        if self.github {
            println!("::group::{title}");
        }

        let start = Instant::now();
        let result = run();
        let duration = start.elapsed();

        if self.github {
            println!("::endgroup::");
        }

        let status = match result {
            Ok(()) => Status::Passed,
            Err(e) if self.github => {
                println!(
                    "::error title={}::{}",
                    escape(&title, true),
                    escape(&e.description, false)
                );
                Status::Failed
            }
            Err(e) => {
                eprintln!("[{title}] Error: {}", e.description);
                Status::Failed
            }
        };
        self.push(project, task, status, duration);
        status == Status::Passed
    }

    pub(crate) fn skip(&mut self, project: &str, task: Task) {
        self.push(project, task, Status::Skipped, Duration::ZERO);
    }

    pub(crate) fn proven(&mut self, project: &str, task: Task) {
        self.push(project, task, Status::Proven, Duration::ZERO);
    }

    fn push(&mut self, project: &str, task: Task, status: Status, duration: Duration) {
        self.entries.push(Entry {
            project: project.to_string(),
            check: task.to_string(),
            status,
            duration,
        });
    }

    /// Prints the results (and writes the job summary), fails if any check failed.
    pub(crate) fn finish(self, platform: &impl Platform) -> Result<(), CIError> {
        print!("\n{}", self.text());

        if self.github
            && let Some(path) = platform.env().var("GITHUB_STEP_SUMMARY")
        {
            platform
                .fs()
                .open_write_options(path, OpenWriteOptions::new().append().create())?
                .write_all(self.markdown().as_bytes())?;
        }

        let mut failed: Vec<&str> = self
            .entries
            .iter()
            .filter(|e| e.status == Status::Failed)
            .map(|e| e.project.as_str())
            .collect();
        failed.dedup();

        if failed.is_empty() {
            Ok(())
        } else {
            Err(format!("checks failed: {}", failed.join(", ")).into())
        }
    }

    fn text(&self) -> String {
        let titles: Vec<String> = self
            .entries
            .iter()
            .map(|e| format!("{}: {}", e.project, e.check))
            .collect();
        let width = titles.iter().map(String::len).max().unwrap_or(0);

        self.entries
            .iter()
            .zip(titles)
            .map(|(e, title)| {
                let status = format!("{:?}", e.status).to_lowercase();
                let line = format!("{title:width$}  {status:7}  {}", time(e));
                format!("{}\n", line.trim_end())
            })
            .collect()
    }

    fn markdown(&self) -> String {
        let mut markdown = String::from(
            "### agci check\n\n| Project | Check | Result | Time |\n| --- | --- | --- | --- |\n",
        );

        for e in &self.entries {
            let status = match e.status {
                Status::Passed => "✅ passed",
                Status::Failed => "❌ failed",
                Status::Skipped => "⏭️ skipped",
                Status::Proven => "☑️ proven",
            };
            markdown += &format!(
                "| {} | `{}` | {status} | {} |\n",
                e.project,
                e.check,
                time(e)
            );
        }

        markdown + "\n"
    }
}

fn time(entry: &Entry) -> String {
    let secs = entry.duration.as_secs();

    match entry.status {
        Status::Skipped | Status::Proven => String::new(),
        _ if secs >= 60 => format!("{}m {:02}s", secs / 60, secs % 60),
        _ => format!("{:.1}s", entry.duration.as_secs_f64()),
    }
}

/// Workflow command escaping, properties (e.g. `title`) additionally escape `:` and `,`.
fn escape(text: &str, property: bool) -> String {
    let text = text
        .replace('%', "%25")
        .replace('\r', "%0D")
        .replace('\n', "%0A");

    if property {
        text.replace(':', "%3A").replace(',', "%2C")
    } else {
        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report() -> Report {
        let mut report = Report {
            github: false,
            entries: Vec::new(),
        };
        report.push("agdb", Task::Setup, Status::Passed, Duration::from_secs(4));
        report.push(
            "agdb",
            Task::Check("format"),
            Status::Passed,
            Duration::from_millis(1500),
        );
        report.push(
            "agdb",
            Task::Check("test"),
            Status::Failed,
            Duration::from_secs(83),
        );
        report.skip("agdb", Task::Check("doc"));
        report.push(
            "ui",
            Task::Check("build"),
            Status::Passed,
            Duration::from_secs(2),
        );
        report.proven("web", Task::Check("lint"));
        report
    }

    #[test]
    fn text_aligns_the_results() {
        assert_eq!(
            report().text(),
            "agdb: setup   passed   4.0s
agdb: format  passed   1.5s
agdb: test    failed   1m 23s
agdb: doc     skipped
ui: build     passed   2.0s
web: lint     proven
"
        );
    }

    #[test]
    fn markdown_table() {
        assert!(report().markdown().contains(
            "| agdb | `test` | ❌ failed | 1m 23s |\n| agdb | `doc` | ⏭️ skipped |  |\n"
        ));
    }

    #[test]
    fn failed_projects_are_listed() {
        assert_eq!(
            report()
                .finish(&agplatform::test_platform::test_platform())
                .unwrap_err()
                .description,
            "checks failed: agdb"
        );
    }

    #[test]
    fn escapes_workflow_commands() {
        assert_eq!(escape("a: b, 100%\nc", true), "a%3A b%2C 100%25%0Ac");
        assert_eq!(escape("a: b\r\n", false), "a: b%0D%0A");
    }
}
