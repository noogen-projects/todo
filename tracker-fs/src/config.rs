use std::collections::BTreeMap;
use std::hash::Hash;
use std::io;
use std::path::{Path, PathBuf};

use fs_err as fs;
use indexmap::IndexSet;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use walkdir::WalkDir;

use crate::Placement;

#[derive(Debug, Error)]
pub enum LoadConfigError {
    #[error("{0}")]
    FailToRead(#[from] io::Error),

    #[error("Fail to deserialize: {0}")]
    FailToDeserialize(#[from] toml::de::Error),
}

#[derive(Debug, Error)]
pub enum SaveConfigError {
    #[error("{0}")]
    FailToWrite(#[from] io::Error),

    #[error("Fail to serialize: {0}")]
    FailToSerialize(#[from] toml::ser::Error),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectConfig<ID: Hash + Eq = String> {
    pub id: ID,

    pub name: Option<String>,

    pub desc: Option<String>,

    pub root_dir: Option<PathBuf>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    pub start_id: Option<u64>,

    #[serde(default = "Default::default")]
    #[serde(skip_serializing_if = "IndexSet::is_empty")]
    pub subprojects: IndexSet<ID>,
}

impl<ID: Hash + Eq> ProjectConfig<ID> {
    const MD_BLOCK_PREFIX: &'static str = "```toml project";
    const MD_BLOCK_SUFFIX: &'static str = "```";

    pub fn new(id: ID) -> Self {
        Self {
            id,
            name: None,
            desc: None,
            root_dir: None,
            tags: Default::default(),
            start_id: None,
            subprojects: Default::default(),
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }
}

impl<ID: for<'a> Deserialize<'a> + Hash + Eq> ProjectConfig<ID> {
    pub fn from_toml(content: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(content)
    }

    pub fn load(source: &Placement<impl AsRef<Path>>) -> Result<Self, LoadConfigError> {
        match source {
            Placement::WholeFile(path) => {
                let content = fs::read_to_string(path.as_ref())?;
                Ok(Self::from_toml(&content)?)
            },
            Placement::CodeBlockInFile(path) => {
                let mut content = String::new();
                let file_content = fs::read_to_string(path.as_ref())?;

                let mut in_block = false;
                for line in file_content.lines() {
                    if line.trim().starts_with('`') && line.trim().to_lowercase() == Self::MD_BLOCK_PREFIX {
                        in_block = true;
                    } else if line.trim().starts_with('`') && line.trim().to_lowercase() == Self::MD_BLOCK_SUFFIX {
                        in_block = false;
                    } else if in_block {
                        content.push_str(line);
                        content.push('\n');
                    }
                }

                Self::from_toml(content.trim_end()).map_err(Into::into)
            },
        }
    }
}

pub trait SerializedId: Serialize + Hash + Eq {}
impl<T> SerializedId for T where T: Serialize + Hash + Eq {}

impl<ID: SerializedId> ProjectConfig<ID> {
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string(self)
    }

    pub fn save(&self, destination: Placement<impl AsRef<Path>>) -> Result<(), SaveConfigError> {
        match destination {
            Placement::WholeFile(path) => {
                let content = self.to_toml()?;
                fs::write(path, content).map_err(Into::into)
            },
            Placement::CodeBlockInFile(path) => {
                let content = format!(
                    "{prefix}\n{config}\n{suffix}\n",
                    prefix = Self::MD_BLOCK_PREFIX,
                    config = self.to_toml()?.trim_end(),
                    suffix = Self::MD_BLOCK_SUFFIX
                );

                if path.as_ref().exists() {
                    let file_content = fs::read_to_string(path.as_ref())?;

                    let mut new_content = String::new();
                    let mut inserted = false;
                    let mut in_block = false;
                    for line in file_content.lines() {
                        if line.trim().starts_with('`') && line.trim().to_lowercase() == Self::MD_BLOCK_PREFIX {
                            in_block = true;
                            new_content.push_str(&content);
                            inserted = true;
                        }
                        if !in_block {
                            new_content.push_str(line);
                            new_content.push('\n');
                        }
                        if line.trim().starts_with('`') && line.trim().to_lowercase() == Self::MD_BLOCK_SUFFIX {
                            in_block = false;
                        }
                    }
                    if !inserted {
                        new_content.push_str(&content);
                    }

                    fs::write(path, new_content).map_err(Into::into)
                } else {
                    let project_name = self.name.as_deref().unwrap_or("");
                    fs::write(path, format!("# {project_name}\n\n{content}")).map_err(Into::into)
                }
            },
        }
    }
}

pub fn find_projects<ID: for<'a> Deserialize<'a> + Hash + Eq + Ord + Clone>(
    search_roots: impl IntoIterator<Item = impl AsRef<Path>>,
    project_config_placement_relative: Placement<impl AsRef<Path> + Clone>,
) -> BTreeMap<ID, ProjectConfig<ID>> {
    let mut projects = BTreeMap::new();

    for root in search_roots {
        for entry in WalkDir::new(root).follow_links(true).into_iter().filter_map(Result::ok) {
            let path = entry.path();
            let project_config_placement =
                project_config_placement_relative.map_ref(|relative_path| path.join(relative_path.as_ref()));

            if project_config_placement.as_ref().exists() {
                if let Ok(mut project_config) = ProjectConfig::<ID>::load(&project_config_placement) {
                    if project_config.root_dir.is_none() {
                        project_config.root_dir = Some(path.to_owned());
                    }
                    projects.insert(project_config.id.clone(), project_config);
                }
            }
        }
    }

    projects
}
