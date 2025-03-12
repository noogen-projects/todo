use clap::Parser;
use indexmap::{IndexMap, IndexSet};
use opts::{AddIssue, NewProject};
use todo_app::config::{ConfigLoader, DisplayProjectConfig, SourceConfig, Title};
use todo_app::open_tracker;
use todo_lib::plan::Step;
use todo_tracker_fs::FsTracker;

use crate::opts::{CliOpts, Cmd, Command};

mod command;
mod opts;
mod output;

fn main() -> anyhow::Result<()> {
    let CliOpts { config_file, command } = CliOpts::parse();

    let profile = ConfigLoader::default()
        .maybe_with_config_file(config_file)
        .load()
        .expect("Wrong config structure");

    match command {
        Command::New(NewProject {
            with_manifest,
            with_project_config,
            location,
        }) => {
            let use_manifest = use_manifest(with_manifest, with_project_config, &profile.config.source);
            command::new_project(use_manifest, location, &profile.config)?;
        },
        Command::Add(AddIssue { location, order, issue }) => {
            command::add_issue(location, order, issue, &profile.config)?;
        },
        Command::Default(Cmd::List { project }) | Command::Issue(Cmd::List { project }) => {
            if let Some(project) = project {
                let tracker = open_tracker(&profile.config)?;

                if let Some(project) = tracker.projects().get(&project) {
                    display_steps(&tracker, project.id(), Default::default, None, true, false);
                }
            }
        },
        Command::Project(cmd @ Cmd::Info { .. } | cmd @ Cmd::List { .. }) => {
            let tracker = open_tracker(&profile.config)?;
            let config = &profile.config.display.project;
            let subprojects = tracker.subprojects();

            for project in tracker.projects().values() {
                if project.parent_id().is_none() {
                    let title = if matches!(config.title, Title::IdName) && project.name().is_empty() {
                        Title::Id
                    } else {
                        config.title
                    };

                    match title {
                        Title::Id => outln!("{}", project.id()),
                        Title::Name => outln!("{}", project.name()),
                        Title::IdName => outln!("{id}: {name}", id = project.id(), name = project.name()),
                    };

                    if let Some(children) = subprojects.get(project.id()) {
                        display_subprojects(
                            &tracker,
                            config,
                            children,
                            &subprojects,
                            1,
                            matches!(cmd, Cmd::Info { .. }),
                        );
                    }
                }
            }
        },
        Command::Default(Cmd::Info { .. }) => {
            let tracker = open_tracker(&profile.config)?;

            for project in tracker.projects().values() {
                outln!("{id}", id = project.id());
            }
        },
        _ => {},
    }

    Ok(())
}

fn use_manifest(with_manifest: bool, with_project_config: bool, config: &SourceConfig) -> bool {
    with_manifest || (!with_project_config && config.use_manifest_file_by_default)
}

fn display_steps(
    tracker: &FsTracker,
    project_id: &String,
    print_prefix: impl Fn(),
    max_count: Option<usize>,
    display_substeps: bool,
    compact: bool,
) {
    let max_count = max_count.unwrap_or(usize::MAX);
    if let Some(project_issues) = tracker.project_issues(project_id) {
        let mut parent_ids = Vec::new();

        let mut step_count = 0;
        let mut print_line = |level, text: &str| {
            if step_count < max_count {
                print_prefix();
                outln!("{:1$}{text}", "", level * 2);
            }
            step_count += 1;
        };

        for step in project_issues.steps() {
            match step {
                Step::Issue(id) => {
                    if let Some(issue) = project_issues.get_issue(id) {
                        if let Some(parent_id) = issue.parent_id {
                            if display_substeps {
                                loop {
                                    if Some(parent_id) == parent_ids.last().copied() || parent_ids.pop().is_none() {
                                        break;
                                    }
                                }

                                print_line(parent_ids.len() + 1, &format!("- {}", issue.name));
                                parent_ids.push(issue.id);
                            }
                        } else {
                            print_line(0, &format!("- {}", issue.name));
                            parent_ids.clear();
                        }
                    }
                },
                Step::Milestone(id) => {
                    if let Some(milestone) = project_issues.get_milestone(id) {
                        if !compact {
                            print_line(0, "");
                        }
                        print_line(0, &format!("# {}", milestone.name));
                        if !compact {
                            print_line(0, "");
                        }
                    }
                },
            }
        }
        if step_count > max_count {
            print_prefix();
            outln!("..{}", step_count - max_count);
        }
    }
}

fn display_subprojects(
    tracker: &FsTracker,
    config: &DisplayProjectConfig,
    children: &IndexSet<String>,
    subprojects: &IndexMap<String, IndexSet<String>>,
    depth: usize,
    need_display_steps: bool,
) {
    for (idx, child_id) in children.iter().enumerate() {
        let mut prefix = String::new();
        for level in 0..depth {
            prefix.push_str(if level == 0 { "  " } else { "│    " });
        }
        out!("{prefix}");

        let connection = if idx + 1 == children.len() { "└─" } else { "├─" };
        out!("{connection} ");

        let project = tracker.projects().get(child_id);
        match config.title {
            Title::Id => outln!("{child_id}"),
            Title::Name => outln!("{}", project.map(|project| project.name()).unwrap_or(child_id)),
            Title::IdName => {
                out!("{child_id}");
                let name = project.map(|project| project.name()).unwrap_or_default();
                if !name.is_empty() {
                    out!(": {name}");
                }
                outln!();
            },
        };

        let steps_max_count = if need_display_steps { config.steps } else { 0 };
        if steps_max_count > 0 {
            if let Some(project) = project {
                display_steps(
                    tracker,
                    project.id(),
                    || {
                        if subprojects.get(child_id).is_some() {
                            out!("{prefix}│    │ ");
                        } else {
                            out!("{prefix}│    ");
                        }
                    },
                    Some(steps_max_count),
                    false,
                    true,
                );
            }
        }

        if let Some(children) = subprojects.get(child_id) {
            display_subprojects(tracker, config, children, subprojects, depth + 1, need_display_steps);
        }
    }
}
