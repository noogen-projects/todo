use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::path::{Path, PathBuf};
use std::{fs, io};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use walkdir::WalkDir;

#[derive(Debug, Error)]
pub enum LoadConfigError {
    #[error("Fail to read: {0}")]
    FailToRead(#[from] io::Error),

    #[error("Fail to deserialize: {0}")]
    FailToDeserialize(#[from] toml::de::Error),
}

pub trait Load
where
    Self: for<'a> Deserialize<'a>,
{
    fn load(path: impl AsRef<Path>) -> Result<Self, LoadConfigError> {
        let content = fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectConfig<ID: Hash + Eq = String> {
    pub id: ID,
    pub name: Option<String>,
    pub desc: Option<String>,
    pub path: Option<PathBuf>,
    pub tags: Vec<String>,
    #[serde(default = "Default::default")]
    pub projects: HashSet<ID>,
}

impl<ID: for<'a> Deserialize<'a> + Hash + Eq> Load for ProjectConfig<ID> {}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProjectsConfig<ID: Hash + Eq = String> {
    pub projects: HashMap<ID, ProjectConfig<ID>>,
}

impl<ID: for<'a> Deserialize<'a> + Hash + Eq> Load for ProjectsConfig<ID> {}

pub fn find_projects<ID: for<'a> Deserialize<'a> + Hash + Eq + Clone>(
    search_roots: impl IntoIterator<Item = impl AsRef<Path>>,
    project_config_file: impl AsRef<Path>,
) -> HashMap<ID, ProjectConfig<ID>> {
    let mut projects = HashMap::new();

    for root in search_roots {
        for entry in WalkDir::new(root).follow_links(true).into_iter().filter_map(Result::ok) {
            let path = entry.path().to_owned();
            if let Ok(mut project_config) = ProjectConfig::<ID>::load(path.join(project_config_file.as_ref())) {
                if project_config.path.is_none() {
                    project_config.path = Some(path);
                }
                projects.insert(project_config.id.clone(), project_config);
            }
        }
    }

    projects
}
