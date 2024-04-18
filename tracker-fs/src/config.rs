use std::collections::BTreeMap;
use std::hash::Hash;
use std::io;
use std::path::{Path, PathBuf};

use fs_err as fs;
use indexmap::IndexSet;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use walkdir::WalkDir;

#[derive(Debug, Error)]
pub enum LoadConfigError {
    #[error("{0}")]
    FailToRead(#[from] io::Error),

    #[error("Fail to deserialize: {0}")]
    FailToDeserialize(#[from] toml::de::Error),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectConfig<ID: Hash + Eq = String> {
    pub id: ID,
    pub name: Option<String>,
    pub desc: Option<String>,
    pub path: Option<PathBuf>,
    pub tags: Vec<String>,
    pub start_id: Option<u64>,
    #[serde(default = "Default::default")]
    pub projects: IndexSet<ID>,
}

impl<ID: Hash + Eq> ProjectConfig<ID> {
    pub fn new(id: ID) -> Self {
        Self {
            id,
            name: None,
            desc: None,
            path: None,
            tags: Default::default(),
            start_id: None,
            projects: Default::default(),
        }
    }
}

impl<ID: for<'a> Deserialize<'a> + Hash + Eq> ProjectConfig<ID> {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, LoadConfigError> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }
}

pub fn find_projects<ID: for<'a> Deserialize<'a> + Hash + Eq + Ord + Clone>(
    search_roots: impl IntoIterator<Item = impl AsRef<Path>>,
    project_config_file: impl AsRef<Path>,
) -> BTreeMap<ID, ProjectConfig<ID>> {
    let mut projects = BTreeMap::new();

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
