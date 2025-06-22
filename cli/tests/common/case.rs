use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::{fs, io, mem};

use anyhow::anyhow;
use assert_cmd::Command;
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Parser, Tag, TagEnd};
use todo_app::config::{DEFAULT_CONFIG_FILE_NAME, ROOT_CONFIG_ENV_KEY};

use super::cmd::{Cmd, CmdResponse};

pub struct TestSection {
    pub title: String,
    pub cases: Vec<TestCase>,
}

#[derive(Debug, Default)]
pub struct TestCase {
    pub commands: Vec<String>,
    pub cargo_bin_alias: String,
    pub test_dir: Option<PathBuf>,
    pub output: ExpectedOutput,
}

#[derive(Debug, Default)]
pub struct ExpectedOutput {
    pub text: String,
    pub source_path: Option<PathBuf>,
    pub source_line: Option<usize>,
}

enum Multiline {
    ToEndString(&'static str, String),
    WithLinesHasEnd(&'static str, String),
}

impl Deref for Multiline {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::ToEndString(_, string) => string,
            Self::WithLinesHasEnd(_, string) => string,
        }
    }
}

impl DerefMut for Multiline {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::ToEndString(_, string) => string,
            Self::WithLinesHasEnd(_, string) => string,
        }
    }
}

impl From<Multiline> for String {
    fn from(value: Multiline) -> Self {
        match value {
            Multiline::ToEndString(_, string) => string,
            Multiline::WithLinesHasEnd(_, string) => string,
        }
    }
}

impl TestCase {
    pub fn parse(source: impl AsRef<str>, source_path: Option<PathBuf>, source_line: Option<usize>) -> Self {
        let mut commands = Vec::new();
        let mut expected_output = String::new();
        let mut multiline_command: Option<Multiline> = None;

        // Split into commands and expected output
        for mut line in source.as_ref().lines() {
            if let Some(mut command) = multiline_command.take() {
                command.push('\n');

                let is_last_line = match &command {
                    Multiline::ToEndString(end, _) => line.contains(*end),
                    Multiline::WithLinesHasEnd(end, _) => {
                        if line.ends_with(*end) {
                            if line.len() > 1 {
                                line = &line[..line.len() - 1];
                            } else {
                                line = "";
                            }
                            false
                        } else {
                            true
                        }
                    },
                };

                command.push_str(line);
                if is_last_line {
                    commands.push(command.into());
                } else {
                    multiline_command = Some(command);
                }
                continue;
            }

            if line.starts_with("$") {
                let mut line = line.trim_start_matches('$').trim_start().to_string();

                let open_string_idx = line.rfind("#\"");
                let close_string_idx = line.rfind("\"#");
                if open_string_idx.is_some() && open_string_idx.map(|idx| idx + 1) >= close_string_idx {
                    multiline_command = Some(Multiline::ToEndString("\"#", line));
                } else {
                    let mark_string_count = line.matches("\"").count();
                    if mark_string_count % 2 == 1 {
                        multiline_command = Some(Multiline::ToEndString("\"", line));
                    } else if line.ends_with('\\') {
                        line.pop();
                        multiline_command = Some(Multiline::WithLinesHasEnd("\\", line));
                    } else {
                        commands.push(line);
                    }
                }
            } else if !commands.is_empty() {
                expected_output.push_str(line);
                expected_output.push('\n');
            }
        }

        if let Some(command) = multiline_command {
            commands.push(command.into());
        }

        // Remove trailing newline
        if !source.as_ref().ends_with('\n') && expected_output.ends_with('\n') {
            expected_output.pop();
        }

        Self {
            commands,
            cargo_bin_alias: String::new(),
            test_dir: None,
            output: ExpectedOutput {
                text: expected_output,
                source_path,
                source_line,
            },
        }
    }

    pub fn with_cargo_bin_alias(mut self, alias: impl Into<String>) -> Self {
        self.cargo_bin_alias = alias.into();
        self
    }

    pub fn with_test_dir(mut self, test_dir: impl Into<PathBuf>) -> Self {
        self.test_dir = Some(test_dir.into());
        self
    }

    pub fn run(&self) -> anyhow::Result<()> {
        let mut root_dir = self.test_dir.clone().unwrap_or_default();
        if !root_dir.exists() {
            return Err(anyhow!("Root directory `{}` does not exist", root_dir.display()));
        }

        for command in &self.commands {
            match Cmd::parse(&root_dir, &command) {
                Ok(cmd) => match cmd.run()? {
                    CmdResponse::Success => (),
                    CmdResponse::ChangeDirTo(path) => root_dir = path,
                    CmdResponse::Output(output) => self.assert_command_output(&root_dir, command, output),
                },
                Err(parts) => {
                    if let [name, args @ ..] = &parts[..] {
                        let mut cmd = if *name == self.cargo_bin_alias {
                            Command::cargo_bin(env!("CARGO_BIN_EXE_todo"))?
                        } else {
                            Command::cargo_bin(name)?
                        };

                        let cmd_assert = cmd
                            .env(ROOT_CONFIG_ENV_KEY, format!("./{DEFAULT_CONFIG_FILE_NAME}"))
                            .args(args)
                            .current_dir(&root_dir)
                            .assert();

                        let stdout = separate_logs(&String::from_utf8_lossy(&cmd_assert.get_output().stdout));
                        let stderr = separate_logs(&String::from_utf8_lossy(&cmd_assert.get_output().stderr));
                        let full_output = format!("{}{}", stdout, stderr);

                        self.assert_command_output(&root_dir, command, full_output);
                    } else {
                        return Err(anyhow!("Invalid command `{}`", command));
                    }
                },
            }
        }

        Ok(())
    }

    pub fn assert_command_output(&self, root_dir: impl AsRef<Path>, command: impl AsRef<str>, output: impl AsRef<str>) {
        let root_dir = root_dir.as_ref();
        let command = command.as_ref();
        let output = output.as_ref();

        let expected_output = self
            .output
            .text
            .replace("${current_dir_path}", &root_dir.to_string_lossy());

        let source_path = self
            .output
            .source_path
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_default();
        let source_line = self.output.source_line.unwrap_or_default();

        // On macOS, temporary directories may appear with a `/private` prefix,
        // e.g., `/private/var/folders/...`, which causes mismatch with expected output
        // defined as `/var/folders/...`. To ensure cross-platform consistency,
        // we normalize such paths in test output comparison.
        let normalize_path = |s: &str| s.replace("/private/var/", "/var/");

        let normalized_output = normalize_path(output);
        let normalized_expected = normalize_path(&expected_output);

        assert_eq!(
            normalized_output, normalized_expected,
            "Command `{command}` in source {source_path}:{source_line}"
        );
    }
}

pub fn parse_tests_from_markdown(
    md_file_path: impl AsRef<Path>,
    cargo_bin_alias: Option<impl Into<String> + Clone>,
) -> io::Result<Vec<TestSection>> {
    let md_file_path = md_file_path.as_ref();
    let content = fs::read_to_string(md_file_path)?;
    let parser = Parser::new(&content);

    let mut sections = Vec::new();
    let mut cases = Vec::new();
    let mut test_case = None;
    let mut test_case_start_line = None;
    let mut section_title = String::new();
    let mut in_test_case_code_block = false;
    let mut in_section_heading = false;

    for (event, range) in parser.into_offset_iter() {
        match event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang)))
                if lang.as_ref() == "sh" || lang.as_ref() == "shell" =>
            {
                in_test_case_code_block = true;
                test_case_start_line = Some(content.split_at(range.start).0.lines().count() + 1);
            },
            Event::Text(text) if in_test_case_code_block => {
                let new_test_case = TestCase::parse(text, Some(md_file_path.into()), test_case_start_line);
                test_case = Some(if let Some(alias) = cargo_bin_alias.clone() {
                    new_test_case.with_cargo_bin_alias(alias)
                } else {
                    new_test_case
                });
            },
            Event::End(TagEnd::CodeBlock) if in_test_case_code_block => {
                if let Some(test) = test_case.take() {
                    cases.push(test);
                }
                in_test_case_code_block = false;
            },
            Event::Start(Tag::Heading {
                level: HeadingLevel::H1,
                ..
            }) => {
                if !cases.is_empty() {
                    sections.push(TestSection {
                        title: mem::take(&mut section_title),
                        cases,
                    });
                    cases = Vec::new();
                }
                in_section_heading = true;
            },
            Event::Text(text) if in_section_heading => {
                section_title = text.to_string();
            },
            Event::End(TagEnd::Heading(HeadingLevel::H1)) if in_section_heading => {
                in_section_heading = false;
            },
            _ => {},
        }
    }

    if !cases.is_empty() {
        sections.push(TestSection {
            title: section_title,
            cases,
        });
    }

    Ok(sections)
}

fn separate_logs(source: &str) -> String {
    let mut outputs = source
        .lines()
        .filter_map(|line| {
            if line.trim().starts_with("[log]") {
                println!("{line}");
                None
            } else {
                Some(line)
            }
        })
        .collect::<Vec<_>>();

    if source.ends_with('\n') {
        outputs.push("");
    }

    outputs.join("\n")
}
