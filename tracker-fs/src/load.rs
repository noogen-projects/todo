use std::hash::Hash;
use std::io::{self, BufRead};
use std::path::Path;
use std::str::FromStr;

use fs_err as fs;
use indexmap::IndexMap;
use todo_lib::issue::Issue;
use todo_lib::plan::Plan;
use todo_lib::project::Project;

use self::parse::Item;
use crate::config::ProjectConfig;
use crate::generator::IdGenerator;

pub mod parse;

pub fn project<ID: Hash + Eq>(id: ID, parent: Option<ID>, config: ProjectConfig<ID>) -> Project<ID> {
    let mut project = Project::new(id, config.name.unwrap_or_default()).with_subprojects(config.subprojects);
    if let Some(parent_id) = parent {
        project.set_parent(parent_id);
    }
    project
}

#[derive(Debug, Clone)]
pub enum Source<P> {
    WholeFile(P),
    CodeBlockInFile(P),
}

impl<P> AsRef<P> for Source<P> {
    fn as_ref(&self) -> &P {
        match self {
            Self::WholeFile(p) => p,
            Self::CodeBlockInFile(p) => p,
        }
    }
}

pub fn project_plan<ID, GEN>(source: Source<impl AsRef<Path>>, id_generator: GEN) -> Option<io::Result<Plan<ID>>>
where
    ID: Hash + Eq + Clone + FromStr,
    GEN: IdGenerator<Id = ID> + Copy,
{
    if source.as_ref().as_ref().exists() {
        Some(match source {
            Source::WholeFile(path) => fs::File::open(path.as_ref())
                .and_then(|file| plan_from_lines(io::BufReader::new(file).lines().enumerate(), id_generator)),
            Source::CodeBlockInFile(path) => fs::File::open(path.as_ref()).and_then(|file| {
                let mut in_block = false;
                let mut inner_blocks: usize = 0;
                let lines = io::BufReader::new(file).lines().enumerate().filter(|(_, line)| {
                    let Ok(line) = line else {
                        return false;
                    };

                    if !in_block {
                        let prefix = "```md todo";
                        if let Some(start) = line.get(..line.len().min(prefix.len() + 1)) {
                            if start.trim().to_lowercase() == prefix {
                                in_block = true;
                            }
                        }
                        false
                    } else {
                        let line = line.trim_end();
                        if line.starts_with("```") {
                            if line.chars().nth(3).map(|ch| !ch.is_whitespace()).unwrap_or(false) {
                                inner_blocks += 1;
                            } else if inner_blocks == 0 {
                                in_block = false;
                            } else {
                                inner_blocks -= 1;
                            }
                        }
                        in_block
                    }
                });
                plan_from_lines(lines, id_generator)
            }),
        })
    } else {
        None
    }
}

pub fn plan_from_lines<ID, GEN>(
    lines: impl IntoIterator<Item = (usize, io::Result<String>)>,
    id_generator: GEN,
) -> io::Result<Plan<ID>>
where
    ID: Hash + Eq + Clone + FromStr,
    GEN: IdGenerator<Id = ID> + Copy,
{
    let mut planned = Plan::<ID>::new();
    let mut last = Last::<ID>::new();

    for (line_idx, line) in lines {
        let line = line?;

        match Item::parse(line, id_generator) {
            (Item::Empty, _) => {
                if let Line::Separator = last.line {
                    planned.add_issues(last.extract_issues());
                }
                last.line = Line::Empty;
            },
            (Item::Separator, _) => {
                if let Line::Empty = last.line {
                    last.line = Line::Separator;
                } else {
                    last.line = Line::Other;
                }
            },
            (Item::Issue(mut issue), issue_level) => {
                if issue_level == last.issue_level {
                    if let (Line::Issue | Line::Description | Line::Empty, Some(id)) =
                        (last.line, last.issue_parent_id.clone())
                    {
                        last.parsed_issues
                            .get_mut(&id)
                            .expect("issue for previous parent must parsed")
                            .subissues
                            .insert(issue.id.clone());
                        issue.parent_id = Some(id);
                    }
                } else if issue_level != 0 {
                    let parent_issue = if issue_level == last.issue_level + 1 {
                        last.parsed_issues.last_mut().map(|(_, last_issue)| last_issue)
                    } else if issue_level < last.issue_level {
                        last.find_parent(issue_level)
                    } else {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!(
                                "in line {line_idx}: issue level = {issue_level} is greater than previous issue level + 1 = {}",
                                last.issue_level + 1
                            ),
                        ));
                    }.ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!(
                                "in line {line_idx}: parent not found for issue level = {issue_level}"
                            ),
                        )
                    })?;

                    parent_issue.subissues.insert(issue.id.clone());
                    issue.parent_id = Some(parent_issue.id.clone());
                }

                last.insert_issue(issue, issue_level);
                last.line = Line::Issue;
            },
            (Item::Milestone(mut milestone), _milestone_level) => {
                milestone.needed_issues.extend(last.parsed_issues.keys().cloned());
                planned.add_issues(last.extract_issues());
                planned.add_milestone(milestone);
                last.line = Line::Milestone;
            },
            (Item::Text(text), text_level) => match last.line {
                Line::Issue | Line::Description | Line::Empty if text_level > last.issue_level => {
                    let mut padding = (last.issue_level + 1) * 2;
                    let description_line = text.trim_start_matches(|ch| {
                        if padding > 0 && ch == ' ' {
                            true
                        } else {
                            padding -= 1;
                            false
                        }
                    });

                    let target_issue = last
                        .parsed_issues
                        .last_mut()
                        .expect("issue for description must exist")
                        .1;
                    if !target_issue.content.is_empty() {
                        target_issue.content.push('\n');
                    }
                    target_issue.content.push_str(description_line);

                    last.line = Line::Description;
                },
                _ => last.line = Line::Other,
            },
        }
    }
    planned.add_issues(last.extract_issues());

    Ok(planned)
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum Line {
    #[default]
    None,
    Empty,
    Separator,
    Issue,
    Description,
    Milestone,
    Other,
}

#[derive(Debug, Default)]
struct Last<ID> {
    issue_level: usize,
    issue_parent_id: Option<ID>,
    line: Line,
    parsed_issues: IndexMap<ID, Issue<ID>>,
}

impl<ID: Hash + Eq + PartialEq + Clone> Last<ID> {
    fn new() -> Self {
        Self {
            issue_level: 0,
            issue_parent_id: None,
            line: Line::None,
            parsed_issues: IndexMap::new(),
        }
    }

    fn find_parent(&mut self, item_level: usize) -> Option<&mut Issue<ID>> {
        let diff = self.issue_level - item_level;

        let mut parent_issue_idx = self.parsed_issues.get_index_of(self.issue_parent_id.as_ref()?)?;
        for _ in 0..diff {
            let parent_issue = self.parsed_issues.get_index(parent_issue_idx)?.1;
            parent_issue_idx = self.parsed_issues.get_index_of(parent_issue.parent_id.as_ref()?)?;
        }
        Some(self.parsed_issues.get_index_mut(parent_issue_idx)?.1)
    }

    fn extract_issues(&mut self) -> impl IntoIterator<Item = Issue<ID>> + '_ {
        self.parsed_issues.drain(..).map(|(_, issue)| issue)
    }

    fn insert_issue(&mut self, issue: Issue<ID>, issue_level: usize) {
        self.issue_parent_id.clone_from(&issue.parent_id);
        self.issue_level = issue_level;
        self.parsed_issues.insert(issue.id.clone(), issue);
    }
}
