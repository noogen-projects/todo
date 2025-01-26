use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Context;
use todo_app::config::Config;
use todo_tracker_fs::config::ProjectConfig;
use todo_tracker_fs::{save, Target};

use crate::{eoutln, outln};

pub fn new_project(
    using_manifest: bool,
    path: impl Into<PathBuf> + AsRef<Path>,
    config: &Config,
) -> anyhow::Result<()> {
    let path_ref = path.as_ref();

    let Some(name) = name_from_path(path_ref) else {
        eoutln!(
            "Error: path `{}` is not containing a directory name",
            path_ref.display()
        );
        return Ok(());
    };

    let is_project_name_only = path_ref.iter().count() == 1;
    let full_path = if path_ref.is_relative() {
        std::env::current_dir()?.join(path)
    } else {
        path.into()
    };

    if full_path.exists() {
        eoutln!("Error: destination `{}` already exists", full_path.display());
        return Ok(());
    }

    if is_project_name_only {
        outln!("    Creating `{name}` project");
    } else {
        let parent_path = full_path.parent().context("Full path must have a parent")?;
        outln!("    Creating `{name}` project under `{}`", parent_path.display());
    }

    fs::create_dir(&full_path)
        .with_context(|| format!("Failed to create project directory {}", full_path.display()))?;

    let destination = if using_manifest {
        let filename = config
            .source
            .manifest_filename_regex
            .replace(&config.source.manifest_filename_example, &name);
        Target::CodeBlockInFile(full_path.join(filename.as_ref()))
    } else {
        Target::WholeFile(full_path.join(&config.source.project_config_file))
    };

    let project_config = ProjectConfig::new(name.clone()).with_name(name);
    save::project(&project_config, destination)?;

    Ok(())
}

fn name_from_path(path: impl AsRef<Path>) -> Option<String> {
    path.as_ref().file_name().map(|name| name.to_string_lossy().into())
}
