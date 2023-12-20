use std::path::PathBuf;

use clap::{Parser, Subcommand};
use indexmap::{IndexMap, IndexSet};
use todo_app::config::ConfigLoader;
use todo_app::open_tracker;

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

            let subprojects = tracker.subprojects();
            for project in tracker.projects().values() {
                if project.parent_id().is_none() {
                    println!("{id}", id = project.id());
                    if let Some(children) = subprojects.get(project.id()) {
                        print_subprojects(children, &subprojects, 1);
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

fn print_subprojects<'a>(
    children: &IndexSet<String>,
    subprojects: &'a IndexMap<String, IndexSet<String>>,
    depth: usize,
) {
    for (idx, child_id) in children.iter().enumerate() {
        for level in 0..depth {
            print!("{}", if level == 0 { "  " } else { "│    " });
        }

        let connection = if idx + 1 == children.len() { "└─" } else { "├─" };
        println!("{connection} {child_id}");

        if let Some(children) = subprojects.get(child_id) {
            print_subprojects(children, subprojects, depth + 1);
        }
    }
}
