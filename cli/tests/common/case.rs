use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::{fs, io, mem};

use anyhow::{anyhow, Context};
use assert_cmd::Command;
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Parser, Tag, TagEnd};
use regex::Regex;

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

impl TestCase {
    pub fn parse(source: impl AsRef<str>, source_path: Option<PathBuf>, source_line: Option<usize>) -> Self {
        let mut commands = Vec::new();
        let mut expected_output = String::new();

        // Split into commands and expected output
        for line in source.as_ref().lines() {
            if line.starts_with("$ ") {
                commands.push(line[2..].to_string());
            } else if !commands.is_empty() {
                expected_output.push_str(line);
                expected_output.push('\n');
            }
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

    pub fn with_root_dir(mut self, root_dir: impl Into<PathBuf>) -> Self {
        self.test_dir = Some(root_dir.into());
        self
    }

    pub fn run(&self) -> anyhow::Result<()> {
        let mut root_dir = self.test_dir.clone().unwrap_or_default();
        if !root_dir.exists() {
            return Err(anyhow!("Root directory `{}` does not exist", root_dir.display()));
        }

        for command in &self.commands {
            let command_chunks = split_command_line(command);

            match &command_chunks[..] {
                ["mkdir", pathes @ ..] => {
                    for path in pathes {
                        let dir = root_dir.join(path);
                        fs::create_dir_all(&dir)
                            .with_context(|| format!("Failed to create directory `{}`", dir.display()))?;
                    }
                },
                ["cd", path] => {
                    let dir = root_dir.join(path);
                    Command::new("cd").arg(&dir).assert().success();
                    root_dir = dir;
                },
                [name, args @ ..] => {
                    let mut cmd = if *name == self.cargo_bin_alias {
                        Command::cargo_bin(env!("CARGO_PKG_NAME"))?
                    } else {
                        Command::new(name)
                    };

                    let cmd_assert = cmd.args(args).current_dir(&root_dir).assert();

                    let stdout = String::from_utf8_lossy(&cmd_assert.get_output().stdout);
                    let stderr = String::from_utf8_lossy(&cmd_assert.get_output().stderr);
                    let full_output = format!("{}{}", stdout, stderr);

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

                    assert_eq!(
                        full_output, expected_output,
                        "Command `{command}` in source {source_path}:{source_line}"
                    );
                },
                _ => return Err(anyhow!("Invalid command `{}`", command)),
            }
        }

        Ok(())
    }
}

fn split_command_line(command: &str) -> Vec<&str> {
    static REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#""([^"]+)"|\S+"#).expect("regex must be correct"));

    REGEX
        .find_iter(command.as_ref())
        .map(|found| found.as_str().trim_matches('"'))
        .collect()
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_test_case() {
        let test = TestCase::parse(
            r#"
$ todo new "test A"
    Creating `test A` project
"#,
            None,
            None,
        );

        assert_eq!(test.commands.len(), 1);
        assert_eq!(test.commands[0], "todo new \"test A\"");
        assert_eq!(test.output.text, "    Creating `test A` project\n");

        let test = TestCase::parse(
            r#"
# Some comment
$ todo new "test A"
    Creating `test A` project"#,
            None,
            None,
        );

        assert_eq!(test.commands.len(), 1);
        assert_eq!(test.commands[0], "todo new \"test A\"");
        assert_eq!(test.output.text, "    Creating `test A` project");

        let test = TestCase::parse(
            r#"
# Some comment

$ todo new "test A"
    Creating `test A` project
error: destination `~/test A` already exists
"#,
            None,
            None,
        );

        assert_eq!(test.commands.len(), 1);
        assert_eq!(test.commands[0], "todo new \"test A\"");
        assert_eq!(
            test.output.text,
            "    Creating `test A` project\nerror: destination `~/test A` already exists\n"
        );
    }

    #[test]
    fn split_commands() {
        assert_eq!(split_command_line("mkdir a b c"), vec!["mkdir", "a", "b", "c"]);
        assert_eq!(split_command_line("cd a/b cd \"ef g\""), vec![
            "cd", "a/b", "cd", "ef g"
        ]);
        assert_eq!(split_command_line("echo \"test A\""), vec!["echo", "test A"]);
        assert_eq!(split_command_line("echo a \"b c d\" ef"), vec![
            "echo", "a", "b c d", "ef"
        ]);
    }
}
