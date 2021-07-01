use crate::{Ordering, TestName, TestResult, TestedResult};
use console::style;
use std::{
    fmt,
    io::{self, Write},
    iter::FromIterator,
};

pub struct Progress<O> {
    output: O,
    indent: usize,
}

#[derive(Debug)]
pub enum Report {
    Grouped {
        reports: Vec<Report>,
    },
    Test {
        name: TestName,
        result: TestResult,
    },
    Group {
        name: TestName,
        ordering: Ordering,
        reports: Vec<Report>,
    },
}

impl Progress<std::io::Stdout> {
    pub fn stdout() -> Self {
        Self::new_with(std::io::stdout())
    }
}

impl<O> Progress<O>
where
    O: std::io::Write,
{
    const INDENT_UNIT: &'static str = "";
    const INDENT_GROWTH: usize = 2;
    const TIME_PRECISION: usize = 2;

    pub fn new_with(output: O) -> Self {
        Self { output, indent: 0 }
    }

    fn handle_result(
        &mut self,
        name: impl fmt::Display,
        result: &TestResult,
    ) -> std::io::Result<()> {
        writeln!(
            &mut self.output,
            "{indent:indent_level$}{result:.<7}{duration:.>18} {name}",
            indent = Self::INDENT_UNIT,
            indent_level = self.indent * Self::INDENT_GROWTH,
            duration = format!(
                "{duration:.precision$?}",
                precision = Self::TIME_PRECISION,
                duration = result.duration
            ),
            result = if result.is_success() {
                style(&result.short).green().bright()
            } else if result.is_skipped() {
                style(&result.short).yellow()
            } else if result.is_timeout() {
                style(&result.short).magenta()
            } else {
                style(&result.short).red().bright()
            },
            name = style(name).white().bold(),
        )?;
        if !result.details.is_empty() {
            writeln!(
                &mut self.output,
                "{indent:indent_level$}{details}",
                indent = Self::INDENT_UNIT,
                indent_level = self.indent * Self::INDENT_GROWTH,
                details = style(&result.details).white().dim(),
            )?;
        }

        Ok(())
    }

    fn handle_group_start(&mut self, name: impl fmt::Display) -> std::io::Result<()> {
        writeln!(
            &mut self.output,
            "{indent:indent_level$}{name}",
            indent = Self::INDENT_UNIT,
            indent_level = self.indent * Self::INDENT_GROWTH,
            name = style(name).white().bold(),
        )?;

        self.indent = self.indent.saturating_add(1);

        Ok(())
    }

    fn handle_group_end(&mut self) -> std::io::Result<()> {
        self.indent = self.indent.saturating_sub(1);
        Ok(())
    }

    pub fn handle(&mut self, result: &TestedResult) -> std::io::Result<()> {
        match result {
            TestedResult::Single { name, result } => self.handle_result(name, result),
            TestedResult::GroupStart { name, .. } => self.handle_group_start(name),
            TestedResult::GroupEnd { .. } => self.handle_group_end(),
        }
    }
}

impl Report {
    #[inline]
    fn push(&mut self, report: Report) {
        match self {
            Self::Grouped { reports } => reports.push(report),
            Self::Group { reports, .. } => reports.push(report),
            Self::Test { .. } => panic!("we should not try to push a result to a single test"),
        }
    }

    pub fn is_success(&self) -> bool {
        match self {
            Report::Grouped { reports } => reports.iter().all(|r| r.is_success()),
            Report::Test { result, .. } => result.is_success(),
            Report::Group { reports, .. } => reports.iter().all(|r| r.is_success()),
        }
    }

    fn report_<O: Write>(&self, indent: &str, output: &mut O) -> io::Result<()> {
        let next_indent = format!("{current}#", current = indent);
        beard::beard! {
            output,
            if let Report::Test { name, result } = (self) {
                "1. `" { name } "` (" { format!("{:?}", result.duration) } "): **" { &result.short } "**\n"
                if (!result.details.is_empty()) {
                    "   "{ &result.details } "\n"
                }
            }
            if let Report::Grouped { reports } = (self) {
                for report in (reports) {
                    || { report.report_(&next_indent, output)? }
                }
            }
            if let Report::Group { name, reports, .. } = (self) {
                { indent } " " {name} " (" { reports.len() } ")" "\n"
                "\n"
                for report in (reports) {
                    || { report.report_(&next_indent, output)? }
                }
            }
        };
        Ok(())
    }

    pub fn report<O: Write>(&self, output: &mut O) -> io::Result<()> {
        self.report_("#", output)
    }
}

macro_rules! push {
    ($reports:ident, $report:expr) => {
        if let Some(current) = $reports.last_mut() {
            current.push($report)
        } else {
            $reports.push($report)
        }
    };
}

impl FromIterator<TestedResult> for Report {
    fn from_iter<T: IntoIterator<Item = TestedResult>>(results: T) -> Self {
        let mut reports: Vec<Report> = vec![];

        for result in results {
            match result {
                TestedResult::Single { name, result } => {
                    push!(reports, Self::Test { name, result });
                }
                TestedResult::GroupStart { name, ordering } => {
                    let report = Self::Group {
                        name,
                        ordering,
                        reports: Vec::new(),
                    };
                    reports.push(report);
                }
                TestedResult::GroupEnd { name, .. } => {
                    if let Some(report) = reports.pop() {
                        if let Self::Group { name: expected, .. } = &report {
                            assert_eq!(
                                &name,
                                expected,
                                "Group named '{name}' didn't finish with the expected '{expected}'",
                                name = name,
                                expected = expected
                            );
                        }

                        push!(reports, report);
                    } else {
                        panic!(
                            "missing group in our stack for group ending named {name}",
                            name = name
                        )
                    }
                }
            }
        }

        if reports.len() == 1 {
            // safe to unwrap we already checked the length
            reports.pop().unwrap()
        } else {
            Self::Grouped { reports }
        }
    }
}
