use std::hash::Hash;
use std::io::{self, BufRead};
use std::path::Path;
use std::str::FromStr;

use fs_err as fs;
use indexmap::IndexMap;
use todo_lib::issue::{Issue, PlannedIssues};
use todo_lib::project::Project;

use self::parse::Item;
use crate::config::ProjectConfig;
use crate::generator::IdGenerator;

pub mod parse;

pub fn project<ID: Hash + Eq>(id: ID, parent: Option<ID>, config: ProjectConfig<ID>) -> Project<ID> {
    let mut project = Project::new(id, config.name.unwrap_or_default()).with_subprojects(config.projects);
    if let Some(parent_id) = parent {
        project.set_parent(parent_id);
    }
    project
}

pub fn project_planned_issues<ID, GEN>(
    project_root: impl AsRef<Path>,
    id_generator: GEN,
) -> Option<io::Result<PlannedIssues<ID>>>
where
    ID: Hash + Eq + Clone + FromStr,
    GEN: IdGenerator<Id = ID> + Copy,
{
    let path = project_root.as_ref().join("TODO.md");
    if path.exists() {
        Some(
            fs::File::open(path)
                .and_then(|file| planned_issues_from_lines(io::BufReader::new(file).lines().enumerate(), id_generator)),
        )
    } else {
        None
    }
}

pub fn planned_issues_from_lines<ID, GEN>(
    lines: impl IntoIterator<Item = (usize, io::Result<String>)>,
    id_generator: GEN,
) -> io::Result<PlannedIssues<ID>>
where
    ID: Hash + Eq + Clone + FromStr,
    GEN: IdGenerator<Id = ID> + Copy,
{
    let mut planned = PlannedIssues::<ID>::new();
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
                if issue_level == last.level {
                    if let (Line::Issue, Some(id)) = (last.line, last.parent_id.clone()) {
                        last.parsed_issues
                            .get_mut(&id)
                            .expect("issue for previous parent must parsed")
                            .subissues
                            .insert(issue.id.clone());
                        issue.parent_id = Some(id);
                    }

                    last.insert_issue(issue, issue_level);
                } else {
                    let parent_issue = if issue_level == last.level + 1 {
                        last.parsed_issues.last_mut().map(|(_, last_issue)| last_issue)
                    } else if issue_level < last.level {
                        last.find_parent(issue_level)
                    } else {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!(
                                "in line {line_idx}: issue level = {issue_level} is greater than previous issue level + 1 = {}",
                                last.level + 1
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

                    last.insert_issue(issue, issue_level);
                }
                last.line = Line::Issue;
            },
            (Item::Milestone(mut milestone), _milestone_level) => {
                milestone.needed_issues.extend(last.parsed_issues.keys().cloned());
                planned.add_issues(last.extract_issues());
                planned.add_milestone(milestone);
                last.line = Line::Milestone;
            },
            (Item::Text(text), text_level) => match last.line {
                Line::Issue | Line::Description | Line::Empty if text_level > last.level => {
                    let mut padding = last.level * 2;
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
                    target_issue.content.push_str(description_line);
                    target_issue.content.push('\n');

                    last.line = Line::Description;
                },
                _ => last.line = Line::Other,
            },
        }
    }
    planned.add_issues(last.extract_issues());

    Ok(planned)
}

#[derive(Default, Clone, Copy)]
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

#[derive(Default)]
struct Last<ID> {
    level: usize,
    parent_id: Option<ID>,
    line: Line,
    parsed_issues: IndexMap<ID, Issue<ID>>,
}

impl<ID: Hash + Eq + PartialEq + Clone> Last<ID> {
    fn new() -> Self {
        Self {
            level: 0,
            parent_id: None,
            line: Line::None,
            parsed_issues: IndexMap::new(),
        }
    }

    fn find_parent(&mut self, item_level: usize) -> Option<&mut Issue<ID>> {
        let diff = self.level - item_level;
        let mut parent_issue_idx = self.parsed_issues.get_index_of(self.parent_id.as_ref()?)?;
        for _ in 1..diff {
            let parent_issue = self.parsed_issues.get_index(parent_issue_idx)?.1;
            parent_issue_idx = self.parsed_issues.get_index_of(parent_issue.parent_id.as_ref()?)?;
        }
        Some(self.parsed_issues.get_index_mut(parent_issue_idx)?.1)
    }

    fn extract_issues(&mut self) -> impl IntoIterator<Item = Issue<ID>> + '_ {
        self.parsed_issues.drain(..).map(|(_, issue)| issue)
    }

    fn insert_issue(&mut self, issue: Issue<ID>, issue_level: usize) {
        self.parent_id = issue.parent_id.clone();
        self.parsed_issues.insert(issue.id.clone(), issue);
        self.level = issue_level;
    }
}
