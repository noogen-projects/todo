use indexmap::IndexMap;
use todo_tracker_fs::config::{find_projects, LoadConfigError};
use todo_tracker_fs::FsTracker;

use crate::config::Config;

pub mod config;
pub mod project;

pub fn open_tracker(config: &Config) -> Result<FsTracker, LoadConfigError> {
    let mut projects = IndexMap::new();

    if config.project_list.enabled {
        for (project_id, project_config) in &config.project {
            if let Some(load_project_config_result) =
                project_config.load_tracker_project_config(&config.project_config_file)
            {
                let loaded_project_config = load_project_config_result?;
                projects.insert(project_id.clone(), loaded_project_config);
            }
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
