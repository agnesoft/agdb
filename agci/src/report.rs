use crate::CIError;
use std::io::Write;
use std::time::Duration;
use std::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Status {
    Passed,
    Failed,
    Skipped,
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
    pub(crate) fn new() -> Self {
        Self {
            github: std::env::var("GITHUB_ACTIONS").is_ok_and(|v| v == "true"),
            entries: Vec::new(),
        }
    }

    /// Returns whether the check passed.
    pub(crate) fn run(
        &mut self,
        project: &str,
        check: &str,
        run: impl FnOnce() -> Result<(), CIError>,
    ) -> bool {
        let title = format!("{project}: {check}");

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
        self.push(project, check, status, duration);
        status == Status::Passed
    }

    pub(crate) fn skip(&mut self, project: &str, check: &str) {
        self.push(project, check, Status::Skipped, Duration::ZERO);
    }

    fn push(&mut self, project: &str, check: &str, status: Status, duration: Duration) {
        self.entries.push(Entry {
            project: project.to_string(),
            check: check.to_string(),
            status,
            duration,
        });
    }

    /// Prints the results (and writes the job summary), fails if any check failed.
    pub(crate) fn finish(self) -> Result<(), CIError> {
        print!("\n{}", self.text());

        if self.github
            && let Ok(path) = std::env::var("GITHUB_STEP_SUMMARY")
        {
            std::fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(path)?
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
        Status::Skipped => String::new(),
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
        report.push(
            "agdb",
            "format",
            Status::Passed,
            Duration::from_millis(1500),
        );
        report.push("agdb", "test", Status::Failed, Duration::from_secs(83));
        report.skip("agdb", "unchanged files");
        report.push("ui", "build", Status::Passed, Duration::from_secs(2));
        report
    }

    #[test]
    fn text_aligns_the_results() {
        assert_eq!(
            report().text(),
            "agdb: format           passed   1.5s
agdb: test             failed   1m 23s
agdb: unchanged files  skipped
ui: build              passed   2.0s
"
        );
    }

    #[test]
    fn markdown_table() {
        assert!(report().markdown().contains(
            "| agdb | `test` | ❌ failed | 1m 23s |\n| agdb | `unchanged files` | ⏭️ skipped |  |\n"
        ));
    }

    #[test]
    fn failed_projects_are_listed() {
        assert_eq!(
            report().finish().unwrap_err().description,
            "checks failed: agdb"
        );
    }

    #[test]
    fn escapes_workflow_commands() {
        assert_eq!(escape("a: b, 100%\nc", true), "a%3A b%2C 100%25%0Ac");
        assert_eq!(escape("a: b\r\n", false), "a: b%0D%0A");
    }
}
