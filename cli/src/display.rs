use std::fmt::Display;

use todo_app::config::{DisplayProjectConfig, Title};
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
    fn display_project_title(&self, project: &Project<ID>, title: Title);
    fn display_steps_list(&self, project: &Project<ID>, config: &DisplayProjectConfig);
    fn display_projects_list(&self, config: &DisplayProjectConfig);
}

impl<ID: HashedId + Clone + Display> DisplayList<ID> for FsTracker<ID> {
    fn display_project_title(&self, project: &Project<ID>, title: Title) {
        match title {
            Title::Id | Title::IdAndName => out!("["),
            _ => (),
        }

        display_project_title_inner(self, project, title);

        match title {
            Title::Id => out!("]"),
            Title::IdAndName => out!("] {name}", name = project.name()),
            _ => (),
        }
        let steps_count = self
            .project_plan(project.id())
            .map(|plan| plan.steps().len())
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
        let count = self.projects().len();

        if count == 1 {
            outln!("List steps of {count} project");
        } else {
            outln!("List steps of {count} projects");
        }

        for project in self.projects().values() {
            let title = if matches!(config.title, Title::IdAndName) && project.name().is_empty() {
                Title::Id
            } else {
                config.title
            };

            outln!();
            self.display_project_title(project, title);
            outln!();

            self.display_steps_list(project, config);
        }
    }
}

fn display_project_title_inner<ID>(tracker: &FsTracker<ID>, project: &Project<ID>, title: Title)
where
    ID: HashedId + Clone + Display,
{
    if let Some(parent_id) = project.parent_id() {
        if let Some(parent) = tracker.projects().get(parent_id) {
            display_project_title_inner(tracker, parent, title);
            out!("/");
        }
    }
    match title {
        Title::Id | Title::IdAndName => out!("{id}", id = project.id()),
        Title::Name => out!("{name}", name = project.name()),
    }
}
