use std::{env, fs};

use anyhow::{anyhow, Context};
use todo_tracker_fs::config::{ProjectConfig, SerializedId};
use todo_tracker_fs::file::find_by_name_part;

pub use self::data::{FsProjectData, ProjectData};
use crate::config::SourceConfig;
use crate::target::Location;

pub mod data;

pub fn create<ID: SerializedId>(ProjectData::Fs(project_data): ProjectData<ID>) -> anyhow::Result<()> {
    let FsProjectData {
        id,
        name,
        root_dir,
        is_current_dir_parent: _,
        config_placement,
        config,
    } = project_data;
    if root_dir.exists() {
        return Err(anyhow!("destination `{}` already exists", root_dir.display()));
    }

    fs::create_dir(&root_dir).with_context(|| format!("Failed to create project directory {}", root_dir.display()))?;

    if let Some(destination) = config_placement {
        if let Some(project_config) = config {
            project_config.save(destination)?;
        } else if let Some(id) = id {
            let project_config = ProjectConfig::new(id).with_name(name);
            project_config.save(destination)?;
        } else {
            let project_config = ProjectConfig::new(name.clone()).with_name(name);
            project_config.save(destination)?;
        }
    }

    Ok(())
}

pub fn default_location<ID>(config: &SourceConfig) -> anyhow::Result<Location<ID>> {
    let mut current_dir = env::current_dir()?;
    let current_dir_string: String = current_dir.to_string_lossy().into();
    let example_project_name = config.manifest_example_project_name();
    let filename_part = config.manifest_filename_example.replace(example_project_name, "");

    loop {
        let project_config_file = current_dir.join(&config.project_config_file);
        if project_config_file.exists() {
            return Ok(Location::Path(current_dir));
        } else {
            if let Some(_manifest_file) = find_by_name_part(&current_dir, &filename_part) {
                return Ok(Location::Path(current_dir));
            }

            let Some(parent) = current_dir.parent() else {
                break;
            };
            current_dir = parent.to_path_buf();
        }
    }

    Err(anyhow!(
        "could not find `{}` or `{}` in `{current_dir_string}` or any parent directory",
        config.project_config_file.display(),
        config.manifest_filename_example.replace(example_project_name, "*")
    ))
}
