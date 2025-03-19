use std::io;

use indexmap::IndexMap;
use thiserror::Error;
use todo_tracker_fs::config::{find_projects, DeserializedId, LoadConfigError};
use todo_tracker_fs::FsTracker;

use crate::config::Config;
use crate::target::Location;

pub mod config;
pub mod issue;
pub mod project;
pub mod target;

#[derive(Debug, Error)]
pub enum OpenTrackerError {
    #[error("Fail to load config: {0}")]
    LoadConfig(#[from] LoadConfigError),

    #[error("Fail to create tracker: {0}")]
    CreateTracker(#[from] io::Error),
}

pub fn open_tracker<ID>(location: Option<Location<ID>>, config: &Config) -> anyhow::Result<FsTracker<ID>>
where
    ID: DeserializedId + ToString + Ord + Default + Clone + TryFrom<String>,
    <ID as TryFrom<String>>::Error: Into<anyhow::Error>,
{
    let mut projects = IndexMap::new();
    let mut project_id = None;
    let mut project_name = None;
    let mut filled_projects = false;

    match location {
        Some(Location::Path(path)) => {
            let found_projects =
                find_projects::<ID>([path], |path| config.source.find_project_config_placement(path, None));
            projects.extend(found_projects);
            filled_projects = true;
        },
        Some(Location::Id(id)) => {
            project_id = Some(id);
        },
        Some(Location::Name(name)) => {
            project_name = Some(name);
        },
        None => {},
    }

    if !filled_projects && config.list.projects.enabled {
        for (id, project_config) in &config.project {
            let id = id.clone().try_into().map_err(Into::into)?;

            if let Some(project_id) = &project_id {
                if *project_id != id {
                    continue;
                }
            }

            if let Some(loaded_project_config) = project_config.load_fs_project_config(id.clone(), &config.source)? {
                if let Some(project_name) = &project_name {
                    if Some(project_name) != loaded_project_config.name.as_ref() {
                        continue;
                    }
                }

                projects.insert(id, loaded_project_config);
                filled_projects = true;
            }
        }

        if !filled_projects {
            let mut found_projects = {
                let mut dirs = if config.search.projects.enabled {
                    config.search.projects.dirs.clone()
                } else {
                    Default::default()
                };
                if project_id.is_some() || project_name.is_some() {
                    dirs.push(".".into());
                }

                find_projects::<ID>(&dirs, |path| config.source.find_project_config_placement(path, None))
            };

            if let Some(project_id) = project_id {
                if let Some(project_config) = found_projects.swap_remove(&project_id) {
                    projects.insert(project_id, project_config);
                    filled_projects = true;
                }
            }

            if let Some(project_name) = project_name {
                if let Some(id) = found_projects.iter().find_map(|(id, project_config)| {
                    if project_config.name.as_ref() == Some(&project_name) {
                        Some(id.clone())
                    } else {
                        None
                    }
                }) {
                    let project_config = found_projects.swap_remove(&id).unwrap(/* extract found value */);
                    projects.insert(id, project_config);
                    filled_projects = true;
                }
            }

            if !filled_projects {
                projects.extend(found_projects);
            }
        }
    }

    Ok(FsTracker::new(
        projects,
        &config.source.manifest_filename_regex,
        &config.source.issues_filename_regex,
    )?)
}
