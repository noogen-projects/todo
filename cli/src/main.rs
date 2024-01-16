use std::path::PathBuf;

use clap::{Parser, Subcommand};
use indexmap::{IndexMap, IndexSet};
use todo_app::config::{ConfigLoader, DisplayProjectConfig, Title};
use todo_app::open_tracker;
use todo_lib::issue::Step;
use todo_tracker_fs::FsTracker;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct CliOpts {
    #[clap(short, long)]
    config_file: Option<PathBuf>,

    #[clap(short, long)]
    project_dir: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    #[command(flatten)]
    Default(Cmd),

    #[command(subcommand)]
    Issue(Cmd),

    #[command(subcommand)]
    Project(Cmd),
}

#[derive(Subcommand, Clone)]
enum Cmd {
    Info {
        #[clap(short, long)]
        project: Option<String>,
    },
    List {
        #[clap(short, long)]
        project: Option<String>,
    },
    Add,
}

fn main() -> anyhow::Result<()> {
    let CliOpts {
        config_file,
        project_dir,
        command,
    } = CliOpts::parse();

    let profile = ConfigLoader::default()
        .maybe_with_config_file(config_file)
        .maybe_with_project_dir(project_dir)
        .load()
        .expect("Wrong config structure");

    match command {
        Command::Default(Cmd::List { project }) | Command::Issue(Cmd::List { project }) => {
            if let Some(project) = project {
                let tracker = open_tracker(&profile.config)?;

                if let Some(project) = tracker.projects().get(&project) {
                    print_steps(&tracker, project.id(), Default::default, None, true);
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
                        Title::Id => println!("{}", project.id()),
                        Title::Name => println!("{}", project.name()),
                        Title::IdName => println!("{id}: {name}", id = project.id(), name = project.name()),
                    };

                    if let Some(children) = subprojects.get(project.id()) {
                        print_subprojects(
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
                println!("{id}", id = project.id());
            }
        },
        _ => {},
    }

    Ok(())
}

fn print_steps(
    tracker: &FsTracker,
    project_id: &String,
    print_prefix: impl Fn() -> (),
    max_count: Option<usize>,
    display_substeps: bool,
) {
    let max_count = max_count.unwrap_or(usize::MAX);
    if let Some(project_issues) = tracker.project_issues(project_id) {
        let mut parent_ids = Vec::new();

        let mut step_count = 0;
        let mut print_line = |level, text| {
            if step_count < max_count {
                print_prefix();
                println!("{:1$}{text}", "", level * 2);
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

                                print_line(parent_ids.len() + 1, format!("- {}", issue.name));
                                parent_ids.push(issue.id);
                            }
                        } else {
                            print_line(0, format!("- {}", issue.name));
                            parent_ids.clear();
                        }
                    }
                },
                Step::Milestone(id) => {
                    if let Some(milestone) = project_issues.get_milestone(id) {
                        print_line(0, format!("# {}", milestone.name));
                    }
                },
            }
        }
        if step_count > max_count {
            print_prefix();
            println!("..{}", step_count - max_count);
        }
    }
}

fn print_subprojects(
    tracker: &FsTracker,
    config: &DisplayProjectConfig,
    children: &IndexSet<String>,
    subprojects: &IndexMap<String, IndexSet<String>>,
    depth: usize,
    display_steps: bool,
) {
    for (idx, child_id) in children.iter().enumerate() {
        let mut prefix = String::new();
        for level in 0..depth {
            prefix.push_str(if level == 0 { "  " } else { "│    " });
        }
        print!("{prefix}");

        let connection = if idx + 1 == children.len() { "└─" } else { "├─" };
        print!("{connection} ");

        let project = tracker.projects().get(child_id);
        match config.title {
            Title::Id => println!("{child_id}"),
            Title::Name => println!("{}", project.map(|project| project.name()).unwrap_or(child_id)),
            Title::IdName => {
                print!("{child_id}");
                let name = project.map(|project| project.name()).unwrap_or_default();
                if !name.is_empty() {
                    print!(": {name}");
                }
                println!();
            },
        };

        let steps_max_count = display_steps.then(|| config.steps).unwrap_or(0);
        if steps_max_count > 0 {
            if let Some(project) = project {
                print_steps(
                    tracker,
                    project.id(),
                    || {
                        if subprojects.get(child_id).is_some() {
                            print!("{prefix}│    │ ");
                        } else {
                            print!("{prefix}│    ");
                        }
                    },
                    Some(steps_max_count),
                    false,
                );
            }
        }

        if let Some(children) = subprojects.get(child_id) {
            print_subprojects(tracker, config, children, subprojects, depth + 1, display_steps);
        }
    }
}
