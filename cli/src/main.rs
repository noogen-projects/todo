use clap::Parser;
use opts::{AddIssue, InitProject, List, NewProject};
use todo_app::config::{ConfigLoader, SourceConfig};

use crate::opts::{CliOpts, Command};

mod command;
mod display;
mod opts;

fn main() -> anyhow::Result<()> {
    let CliOpts {
        config_file,
        global,
        command,
    } = CliOpts::parse();

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
        Command::Init(InitProject {
            with_manifest,
            with_project_config,
            location,
        }) => {
            let use_manifest = use_manifest(with_manifest, with_project_config, &profile.config.source);
            command::init_project(use_manifest, location, &profile.config)?;
        },
        Command::Add(AddIssue { location, order, issue }) => {
            command::add_issue(location, order, issue, &profile.config)?;
        },
        Command::List(List {
            max_steps,
            location,
            project_location,
        }) => {
            let mut config = profile.config;
            if let Some(max_steps) = max_steps {
                config.display.project.max_steps = Some(max_steps);
            }

            command::list(location, project_location, global, &config)?;
        },
        _ => {},
    }

    Ok(())
}

fn use_manifest(with_manifest: bool, with_project_config: bool, config: &SourceConfig) -> bool {
    with_manifest || (!with_project_config && config.use_manifest_file_by_default)
}
