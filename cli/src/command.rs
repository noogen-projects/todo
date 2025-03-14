use std::env;

use anyhow::Context;
use todo_app::config::Config;
use todo_app::issue;
use todo_app::project::{self, FsProjectData, ProjectData};
use todo_app::target::Location;

use crate::opts::{Order, ProjectLocation};
use crate::outln;

pub fn new_project(use_manifest: bool, location: impl Into<String>, config: &Config) -> anyhow::Result<()> {
    let location = Location::<String>::from_unknown(location);
    let project_data = FsProjectData::new(location, &config.source, use_manifest)?;
    let name = &project_data.name;

    if project_data.is_current_dir_parent {
        outln!("    Creating `{name}` project");
    } else {
        let parent_path = project_data.root_dir.parent().context("Full path must have a parent")?;
        outln!("    Creating `{name}` project under `{}`", parent_path.display());
    }

    project::create(ProjectData::Fs(project_data))?;
    Ok(())
}

pub fn init_project(use_manifest: bool, location: Option<String>, config: &Config) -> anyhow::Result<()> {
    let current_dir = env::current_dir()?;
    let location = if let Some(location) = location {
        Location::<String>::from_unknown(location)
    } else {
        Location::Path(current_dir.clone())
    };
    let project_data = FsProjectData::new(location, &config.source, use_manifest)?;
    let name = &project_data.name;

    if project_data.is_current_dir_parent || project_data.root_dir == current_dir {
        outln!("    Initializing `{name}` project");
    } else {
        let parent_path = project_data.root_dir.parent().context("Full path must have a parent")?;
        outln!("    Initializing `{name}` project under `{}`", parent_path.display());
    }

    project::init(ProjectData::Fs(project_data), &config.source)?;
    Ok(())
}

pub fn add_issue(
    location: ProjectLocation,
    order: Order,
    name: impl AsRef<str> + Into<String>,
    config: &Config,
) -> anyhow::Result<()> {
    let location = if let Some(location) = location.into_location() {
        location
    } else {
        project::default_location(&config.source)?
    };
    let order = order
        .into_order()
        .unwrap_or_else(|| config.issue.add_order.into_order());

    let mut project_data = FsProjectData::exists(location, &config.source)?;
    project_data.update_from_config()?;

    outln!(
        "    Adding `{}` issue to `{}` project",
        name.as_ref(),
        project_data.name
    );

    issue::add(ProjectData::Fs(project_data), &config.source, order, name, "")?;
    Ok(())
}
