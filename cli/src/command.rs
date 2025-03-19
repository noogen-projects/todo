use std::env;

use anyhow::Context;
use todo_app::config::Config;
use todo_app::project::{self, FsProjectMetadata, ProjectData};
use todo_app::target::Location;
use todo_app::{issue, open_tracker};

use crate::display::DisplayList;
use crate::opts::{Order, ProjectLocation};
use crate::outln;

pub fn new_project(use_manifest: bool, location: impl Into<String>, config: &Config) -> anyhow::Result<()> {
    let location = Location::<String>::from_unknown(location);
    let project_metadata = FsProjectMetadata::new(location, &config.source, use_manifest)?;
    let name = &project_metadata.name;

    if project_metadata.is_current_dir_parent {
        outln!("    Creating `{name}` project");
    } else {
        let parent_path = project_metadata
            .root_dir
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
    let name = &project_metadata.name;

    if project_metadata.is_current_dir_parent || project_metadata.root_dir == current_dir {
        outln!("    Initializing `{name}` project");
    } else {
        let parent_path = project_metadata
            .root_dir
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
    name: impl AsRef<str> + Into<String>,
    config: &Config,
) -> anyhow::Result<()> {
    let location = location
        .into_location()
        .map(Ok)
        .unwrap_or_else(|| project::default_location(&config.source))?;
    let order = order
        .into_order()
        .unwrap_or_else(|| config.issue.add_order.into_order());

    let mut project_metadata = FsProjectMetadata::exists(location, &config.source)?;
    project_metadata.update_from_config()?;

    outln!(
        "    Adding `{}` issue to `{}` project",
        name.as_ref(),
        project_metadata.name
    );

    issue::add(ProjectData::Fs(project_metadata), &config.source, order, name, "")?;
    Ok(())
}

pub fn list(
    location: Option<String>,
    project_location: ProjectLocation,
    global: bool,
    config: &Config,
) -> anyhow::Result<()> {
    let mut location = location
        .map(Location::from_unknown)
        .or_else(|| project_location.into_location());
    if location.is_none() && !global {
        location = Some(project::default_location(&config.source)?);
    }

    let tracker = open_tracker(location, config)?;
    tracker.display_projects_list(&config.display.project);
    Ok(())
}
