use std::collections::HashMap;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use todo_app::config::{Config, ConfigLoader};
use todo_tracker_fs::config::find_projects;
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
    let mut projects = HashMap::new();

    for (project_id, project_config) in &config.project {
        if let Some(load_project_config_result) =
            project_config.load_tracker_project_config(&config.project_config_file)
        {
            let loaded_project_config = load_project_config_result?;
            projects.insert(project_id.clone(), loaded_project_config);
        }
    }

    if config.project_search.enabled {
        projects.extend(find_projects::<String>(
            &config.project_search.dirs,
            &config.project_config_file,
        ));
    }

    Ok(FsTracker::new(projects))
}
