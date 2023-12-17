use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, Subcommand};
use todo_app::config::{Config, ConfigLoader};
use todo_tracker_fs::config::{find_projects, Load, ProjectsConfig};
use todo_tracker_fs::tracker::FsTracker;

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

            for project in tracker.projects().values() {
                print!("{id} : {name}", id = project.id(), name = project.name());
                if let Some(path) = tracker.path(project.id()) {
                    println!("  {}", path.display());
                } else {
                    println!();
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

fn open_tracker(config: &Config) -> anyhow::Result<FsTracker> {
    let projects = find_projects::<String>(&config.project_search_dirs, &config.project_config_file);
    let mut projects_config = if config.projects_config_file.exists() {
        ProjectsConfig::load(&config.projects_config_file)
            .with_context(|| config.projects_config_file.display().to_string())?
    } else {
        ProjectsConfig::default()
    };
    projects_config.projects.extend(projects);

    Ok(FsTracker::new(projects_config))
}
