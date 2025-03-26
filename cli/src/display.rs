use std::fmt::{Display, Write};

use todo_app::config::{DisplayProjectConfig, TitleConsist};
use todo_lib::id::HashedId;
use todo_lib::plan::Step;
use todo_lib::project::Project;
use todo_tracker_fs::FsTracker;

#[macro_export]
macro_rules! out {
    ($($arg:tt)*) => {{
        print!($($arg)*)
    }};
}

#[macro_export]
macro_rules! outln {
    ($($arg:tt)*) => {{
        println!($($arg)*)
    }};
}

pub trait DisplayList<ID> {
    fn format_project_title_key(&self, project: &Project<ID>, title: TitleConsist) -> String;
    fn display_project_title(
        &self,
        project: &Project<ID>,
        title: TitleConsist,
        title_key: Option<impl AsRef<str>>,
        config: &DisplayProjectConfig,
    );
    fn display_steps_list(&self, project: &Project<ID>, config: &DisplayProjectConfig);
    fn display_projects_list(&self, config: &DisplayProjectConfig);
}

impl<ID: HashedId + Clone + Display> DisplayList<ID> for FsTracker<ID> {
    fn format_project_title_key(&self, project: &Project<ID>, consist: TitleConsist) -> String {
        let mut output = String::new();
        format_project_title_key_inner(self, project, consist, &mut output);
        output
    }

    fn display_project_title(
        &self,
        project: &Project<ID>,
        consist: TitleConsist,
        title_key: Option<impl AsRef<str>>,
        config: &DisplayProjectConfig,
    ) {
        if let Some(before) = match consist {
            TitleConsist::Id => &config.title.id_before,
            TitleConsist::Name => &config.title.name_before,
            TitleConsist::IdAndName => &config.title.id_and_name_before,
        } {
            out!("{before}");
        }

        if let Some(title_key) = title_key {
            out!("{}", title_key.as_ref());
        } else {
            out!("{}", self.format_project_title_key(project, consist));
        }

        if let TitleConsist::IdAndName = consist {
            if project.id().to_string() != project.name() {
                if let Some(separator) = &config.title.id_and_name_separator {
                    out!("{separator}");
                }
                out!("{}", project.name());
            }
        }

        if let Some(after) = match consist {
            TitleConsist::Id => &config.title.id_after,
            TitleConsist::Name => &config.title.name_after,
            TitleConsist::IdAndName => &config.title.id_and_name_after,
        } {
            out!("{after}");
        }

        let steps_count = self
            .project_plan(project.id())
            .map(|plan| {
                plan.steps().iter().fold(0_usize, |count, step| {
                    if !config.show_substeps {
                        if let Step::Issue(id) = step {
                            if let Some(issue) = plan.get_issue(id) {
                                if issue.parent_id.is_some() {
                                    return count;
                                }
                            }
                        }
                    }
                    count + 1
                })
            })
            .unwrap_or(0);
        out!(": {steps_count}")
    }

    fn display_steps_list(&self, project: &Project<ID>, config: &DisplayProjectConfig) {
        let max_count = config.max_steps.unwrap_or(usize::MAX);
        if let Some(plan) = self.project_plan(project.id()) {
            let mut parent_ids = Vec::new();

            let mut step_count = 0;
            let mut display_line = |level, text: &str| {
                if step_count < max_count {
                    outln!("{:1$}{text}", "", level * 2);
                }
                step_count += 1;
            };

            for step in plan.steps() {
                match step {
                    Step::Issue(id) => {
                        if let Some(issue) = plan.get_issue(id) {
                            if let Some(parent_id) = issue.parent_id {
                                if config.show_substeps {
                                    loop {
                                        if Some(parent_id) == parent_ids.last().copied() || parent_ids.pop().is_none() {
                                            break;
                                        }
                                    }

                                    display_line(parent_ids.len() + 1, &format!("- {}", issue.name));
                                    parent_ids.push(issue.id);
                                }
                            } else {
                                display_line(0, &format!("- {}", issue.name));
                                parent_ids.clear();
                            }
                        }
                    },
                    Step::Milestone(id) => {
                        if let Some(milestone) = plan.get_milestone(id) {
                            if !config.compact {
                                display_line(0, "");
                            }
                            display_line(0, &format!("# {}", milestone.name));
                            if !config.compact {
                                display_line(0, "");
                            }
                        }
                    },
                }
            }
            if step_count > max_count {
                outln!("..{}", step_count - max_count);
            }
        }
    }

    fn display_projects_list(&self, config: &DisplayProjectConfig) {
        if !config.compact {
            let count = self.projects().len();

            if count == 1 {
                outln!("List steps of {count} project");
            } else {
                outln!("List steps of {count} projects");
            }
        }

        let mut projects = Vec::new();
        for project in self.projects().values() {
            let consist = if matches!(config.title.consist, TitleConsist::IdAndName) && project.name().is_empty() {
                TitleConsist::Id
            } else {
                config.title.consist
            };
            projects.push((self.format_project_title_key(project, consist), consist, project));
        }

        projects.sort_by(|(title_key1, ..), (title_key2, ..)| title_key1.cmp(title_key2));

        let mut is_first_project = true;
        for (title_key, consist, project) in projects {
            if !config.compact || (!is_first_project && config.separate_projects) {
                outln!();
            }

            self.display_project_title(project, consist, Some(title_key), config);
            outln!();
            self.display_steps_list(project, config);

            is_first_project = false;
        }
    }
}

fn format_project_title_key_inner<ID>(
    tracker: &FsTracker<ID>,
    project: &Project<ID>,
    consist: TitleConsist,
    output: &mut String,
) where
    ID: HashedId + Clone + Display,
{
    if let Some(parent_id) = project.parent_id() {
        if let Some(parent) = tracker.projects().get(parent_id) {
            format_project_title_key_inner(tracker, parent, consist, output);
            write!(output, "/").expect("Failed to write to string");
        }
    }
    match consist {
        TitleConsist::Id | TitleConsist::IdAndName => write!(output, "{id}", id = project.id()),
        TitleConsist::Name => write!(output, "{name}", name = project.name()),
    }
    .expect("Failed to write to string");
}
