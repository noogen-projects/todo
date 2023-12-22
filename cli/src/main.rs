use std::path::PathBuf;

use clap::{Parser, Subcommand};
use indexmap::{IndexMap, IndexSet};
use todo_app::config::{ConfigLoader, DisplayProjectConfig, Title};
use todo_app::open_tracker;
use todo_lib::project::Project;

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

#[derive(Subcommand)]
enum Cmd {
    Info,
    List,
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
        Command::Default(Cmd::List) | Command::Issue(Cmd::List) => {},
        Command::Project(Cmd::List) => {
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
                        print_subprojects(tracker.projects(), config, children, &subprojects, 1);
                    }
                }
            }
        },
        Command::Default(Cmd::Info) => {
            let tracker = open_tracker(&profile.config)?;

            for project in tracker.projects().values() {
                println!("{id}", id = project.id());
            }
        },
        _ => {},
    }

    Ok(())
}

fn print_subprojects(
    projects: &IndexMap<String, Project<String>>,
    config: &DisplayProjectConfig,
    children: &IndexSet<String>,
    subprojects: &IndexMap<String, IndexSet<String>>,
    depth: usize,
) {
    for (idx, child_id) in children.iter().enumerate() {
        for level in 0..depth {
            print!("{}", if level == 0 { "  " } else { "│    " });
        }

        let connection = if idx + 1 == children.len() { "└─" } else { "├─" };
        print!("{connection} ");

        match config.title {
            Title::Id => println!("{child_id}"),
            Title::Name => println!(
                "{}",
                projects.get(child_id).map(|project| project.name()).unwrap_or(child_id)
            ),
            Title::IdName => {
                print!("{child_id}");
                let name = projects.get(child_id).map(|project| project.name()).unwrap_or_default();
                if !name.is_empty() {
                    print!(": {name}");
                }
                println!();
            },
        };

        if let Some(children) = subprojects.get(child_id) {
            print_subprojects(projects, config, children, subprojects, depth + 1);
        }
    }
}
