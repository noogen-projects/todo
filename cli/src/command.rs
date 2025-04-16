use std::env;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context};
use todo_app::config::Config;
use todo_app::project::{self, FsProjectMetadata, ProjectData};
use todo_app::target::Location;
use todo_app::{issue, locate_project_config, open_tracker};

use crate::display::DisplayList;
use crate::opts::{Order, ProjectLocation};
use crate::outln;

pub fn new_project(use_manifest: bool, location: impl Into<String>, config: &Config) -> anyhow::Result<()> {
    let location = Location::<String>::from_unknown(location);
    let project_metadata = FsProjectMetadata::new(location, &config.source, use_manifest)?;
    let name = project_metadata.name();

    if project_metadata.is_current_dir_parent() {
        outln!("    Creating `{name}` project");
    } else {
        let parent_path = project_metadata
            .root_dir()
            .parent()
            .context("Full path must have a parent")?;
        outln!("    Creating `{name}` project under `{}`", parent_path.display());
    }

    project::create(ProjectData::Fs(project_metadata))?;
    Ok(())
}

pub fn init_project(use_manifest: bool, location: Option<String>, config: &Config) -> anyhow::Result<()> {
    let current_dir = env::current_dir()?;
    let location = location
        .map(Location::<String>::from_unknown)
        .unwrap_or_else(|| Location::Path(current_dir.clone()));
    let project_metadata = FsProjectMetadata::new(location, &config.source, use_manifest)?;
    let name = project_metadata.name();

    if project_metadata.is_current_dir_parent() || project_metadata.root_dir() == current_dir {
        outln!("    Initializing `{name}` project");
    } else {
        let parent_path = project_metadata
            .root_dir()
            .parent()
            .context("Full path must have a parent")?;
        outln!("    Initializing `{name}` project under `{}`", parent_path.display());
    }

    project::init(ProjectData::Fs(project_metadata), &config.source)?;
    Ok(())
}

pub fn add_issue(
    location: ProjectLocation,
    order: Order,
    issue_name: impl AsRef<str> + Into<String>,
    config: &Config,
) -> anyhow::Result<()> {
    let current_dir = env::current_dir()?;
    let location = location
        .into_location()
        .map(Ok)
        .unwrap_or_else(|| project::default_path(&current_dir, &config.source).map(Location::Path))?;
    let order = order
        .into_order()
        .unwrap_or_else(|| config.issue.add_order.into_order());

    let Some(project_config) = locate_project_config(location, [&current_dir], config)? else {
        return Err(anyhow!(
            "could not find `{}` or `{}`",
            config.source.project_config_file.display(),
            config
                .source
                .manifest_filename_example
                .replace(config.source.manifest_example_project_name(), "*")
        ));
    };
    let config_placement = config.source.find_project_config_placement(
        project_config.root_dir.as_deref().unwrap_or(Path::new("")),
        project_config.name.as_deref(),
    );
    let project_metadata = FsProjectMetadata::default()
        .with_config_placement_maybe(config_placement)
        .with_config(project_config);

    outln!(
        "    Adding `{}` issue to `{}` project",
        issue_name.as_ref(),
        project_metadata.name()
    );

    issue::add(ProjectData::Fs(project_metadata), &config.source, order, issue_name, "")?;
    Ok(())
}

pub fn list(root: Option<String>, project_location: ProjectLocation, config: &Config) -> anyhow::Result<()> {
    let mut location = project_location.into_location();

    let local_search_roots = if let Some(root) = &root {
        vec![PathBuf::from(root)]
    } else if config.working_mode.is_local() {
        let current_dir = env::current_dir()?;

        if location.is_none() {
            let project_root = project::default_path(&current_dir, &config.source)?;
            location = Some(Location::Path(project_root));
        }
        vec![current_dir]
    } else {
        vec![]
    };

    let tracker = open_tracker(location, local_search_roots, config)?;
    tracker.display_projects_list(&config.display.project);

    Ok(())
}
