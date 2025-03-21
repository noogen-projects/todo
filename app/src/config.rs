use std::env;
use std::env::VarError;
use std::path::{Path, PathBuf};

pub use config::ConfigError;
use config::{Environment, File};
use indexmap::{IndexMap, IndexSet};
use regex::Regex;
use serde::{Deserialize, Serialize};
use todo_tracker_fs::config::{DeserializedId, FsProjectConfig};
use todo_tracker_fs::file::{find_by_name_part, find_in_dir_and_parents};
use todo_tracker_fs::Placement;

use crate::issue::Order;

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct SearchConfig {
    pub projects: SearchProjectsConfig,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct SearchProjectsConfig {
    #[serde(default = "SearchProjectsConfig::default_enabled")]
    pub enabled: bool,

    pub dirs: Vec<PathBuf>,
}

impl Default for SearchProjectsConfig {
    fn default() -> Self {
        Self {
            enabled: Self::default_enabled(),
            dirs: Default::default(),
        }
    }
}

impl SearchProjectsConfig {
    pub const fn default_enabled() -> bool {
        true
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct ListConfig {
    pub projects: ListProjectsConfig,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct ListProjectsConfig {
    #[serde(default = "ListProjectsConfig::default_enabled")]
    pub enabled: bool,
}

impl Default for ListProjectsConfig {
    fn default() -> Self {
        Self {
            enabled: Self::default_enabled(),
        }
    }
}

impl ListProjectsConfig {
    pub const fn default_enabled() -> bool {
        true
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct IssueConfig {
    #[serde(default)]
    pub add_order: IssueAddOrder,
}

#[derive(Copy, Clone, Default, Debug, Deserialize, Serialize)]
pub enum IssueAddOrder {
    First,
    #[default]
    Last,
}

impl IssueAddOrder {
    pub fn into_order(self) -> Order {
        match self {
            Self::First => Order::First,
            Self::Last => Order::Last,
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct ProjectConfig {
    pub path: Option<PathBuf>,

    pub subprojects: IndexSet<String>,
}

impl ProjectConfig {
    pub fn load_fs_project_config<ID>(
        &self,
        project_id: ID,
        config: &SourceConfig,
    ) -> anyhow::Result<Option<FsProjectConfig<ID>>>
    where
        ID: DeserializedId + TryFrom<String>,
        <ID as TryFrom<String>>::Error: Into<anyhow::Error>,
    {
        let Some(mut project_path) = self.path.clone() else {
            return Ok(None);
        };

        if let Some(projects_root_dir) = &config.projects_root_dir {
            if project_path.is_relative() {
                project_path = projects_root_dir.join(project_path)
            }
        }

        let Some(config_placement) = config.find_project_config_placement(&project_path, None) else {
            return Ok(None);
        };

        let mut config = match FsProjectConfig::load(&config_placement) {
            Ok(config) => config,
            Err(err) => {
                if project_path.is_dir() {
                    FsProjectConfig::new(project_id)
                } else {
                    return Err(err.into());
                }
            },
        };

        config.root_dir = Some(project_path);
        for subproject_id in &self.subprojects {
            let id = ID::try_from(subproject_id.clone()).map_err(Into::into)?;
            config.subprojects.insert(id);
        }

        Ok(Some(config))
    }
}

#[derive(Copy, Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Title {
    #[default]
    Id,
    Name,
    IdAndName,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct DisplayProjectConfig {
    pub title: Title,
    pub max_steps: Option<usize>,
    pub show_substeps: bool,
    pub compact: bool,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct DisplayConfig {
    pub project: DisplayProjectConfig,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct SourceConfig {
    #[serde(default = "SourceConfig::default_use_manifest_file_by_default")]
    pub use_manifest_file_by_default: bool,

    #[serde(default = "SourceConfig::default_manifest_filename_regex", with = "serde_regex")]
    pub manifest_filename_regex: Regex,

    #[serde(default = "SourceConfig::default_manifest_filename_example")]
    pub manifest_filename_example: String,

    #[serde(default = "SourceConfig::default_issues_filename_regex", with = "serde_regex")]
    pub issues_filename_regex: Regex,

    #[serde(default = "SourceConfig::default_issues_filename_example")]
    pub issues_filename_example: String,

    #[serde(default = "SourceConfig::default_project_config_file")]
    pub project_config_file: PathBuf,

    pub projects_root_dir: Option<PathBuf>,
}

impl Default for SourceConfig {
    fn default() -> Self {
        Self {
            use_manifest_file_by_default: Self::default_use_manifest_file_by_default(),
            manifest_filename_regex: Self::default_manifest_filename_regex(),
            manifest_filename_example: Self::default_manifest_filename_example(),
            issues_filename_regex: Self::default_issues_filename_regex(),
            issues_filename_example: Self::default_issues_filename_example(),
            project_config_file: Self::default_project_config_file(),
            projects_root_dir: None,
        }
    }
}

impl SourceConfig {
    pub const fn default_use_manifest_file_by_default() -> bool {
        false
    }

    pub fn default_manifest_filename_regex() -> Regex {
        Regex::new("(.*)\\.manifest\\.md$").expect("regex mus be correct")
    }

    pub fn default_manifest_filename_example() -> String {
        "example.manifest.md".into()
    }

    pub fn default_issues_filename_regex() -> Regex {
        Regex::new("^TODO\\.md$").expect("regex mus be correct")
    }

    pub fn default_issues_filename_example() -> String {
        "TODO.md".into()
    }

    pub fn default_project_config_file() -> PathBuf {
        "Project.toml".into()
    }

    pub fn manifest_example_project_name(&self) -> &str {
        self.manifest_filename_regex
            .captures(&self.manifest_filename_example)
            .and_then(|captures| captures.get(1))
            .map(|name| name.as_str())
            .unwrap_or_default()
    }

    pub fn project_manifest_file_name(&self, project_name: impl AsRef<str>) -> String {
        let example_project_name = self.manifest_example_project_name();
        if example_project_name.is_empty() {
            self.manifest_filename_example.clone()
        } else {
            self.manifest_filename_example
                .replace(example_project_name, project_name.as_ref())
        }
    }

    pub fn make_manifest_file_path(&self, root_dir: impl AsRef<Path>, project_name: Option<&str>) -> PathBuf {
        let root_dir = root_dir.as_ref();
        if let Some(name) = project_name {
            root_dir.join(self.project_manifest_file_name(name))
        } else {
            let manifest_file_name_part = self.project_manifest_file_name("");
            find_by_name_part(root_dir, &manifest_file_name_part)
                .unwrap_or_else(|| root_dir.join(manifest_file_name_part))
        }
    }

    pub fn issues_example_project_name(&self) -> &str {
        self.issues_filename_regex
            .captures(&self.issues_filename_example)
            .and_then(|captures| captures.get(1))
            .map(|name| name.as_str())
            .unwrap_or_default()
    }

    pub fn project_issues_file_name(&self, project_name: impl AsRef<str>) -> String {
        let example_project_name = self.issues_example_project_name();
        if example_project_name.is_empty() {
            self.issues_filename_example.clone()
        } else {
            self.issues_filename_example
                .replace(example_project_name, project_name.as_ref())
        }
    }

    pub fn make_issues_file_path(&self, root_dir: impl AsRef<Path>, project_name: Option<&str>) -> PathBuf {
        root_dir
            .as_ref()
            .join(self.project_issues_file_name(project_name.unwrap_or_default()))
    }

    pub fn find_project_config_placement(
        &self,
        root_dir: impl AsRef<Path>,
        project_name: Option<&str>,
    ) -> Option<Placement<PathBuf>> {
        let project_config_file = root_dir.as_ref().join(&self.project_config_file);
        if project_config_file.exists() {
            Some(Placement::WholeFile(project_config_file))
        } else {
            let project_manifest_file = self.make_manifest_file_path(root_dir, project_name);
            project_manifest_file
                .exists()
                .then_some(Placement::CodeBlockInFile(project_manifest_file))
        }
    }

    pub fn find_issues_placement(
        &self,
        root_dir: impl AsRef<Path>,
        project_name: Option<&str>,
    ) -> Option<Placement<PathBuf>> {
        let issues_file = self.make_issues_file_path(root_dir.as_ref(), project_name);
        if issues_file.exists() {
            Some(Placement::WholeFile(issues_file))
        } else {
            let manifest_file = self.make_manifest_file_path(root_dir, project_name);
            manifest_file
                .exists()
                .then_some(Placement::CodeBlockInFile(manifest_file))
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    pub source: SourceConfig,

    pub display: DisplayConfig,

    pub search: SearchConfig,

    pub list: ListConfig,

    pub issue: IssueConfig,

    pub project: IndexMap<String, ProjectConfig>,
}

#[derive(Debug, Default)]
pub struct ConfigBuilder {
    config_paths: Vec<PathBuf>,
}

impl ConfigBuilder {
    pub fn with_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.add_path(path);
        self
    }

    pub fn with_paths(mut self, paths: impl Into<Vec<PathBuf>>) -> Self {
        self.set_paths(paths);
        self
    }

    pub fn add_path(&mut self, path: impl Into<PathBuf>) {
        self.config_paths.push(path.into());
    }

    pub fn set_paths(&mut self, paths: impl Into<Vec<PathBuf>>) {
        self.config_paths = paths.into();
    }

    pub fn build(self) -> Result<Config, ConfigError> {
        let mut config_builder = config::Config::builder();
        for path in &self.config_paths {
            config_builder = config_builder.add_source(File::from(path.as_path()))
        }

        // Add in settings from the environment (with a prefix of TODO)
        // Eg. `TODO__HTTP__PORT=8090 ` would set the `http.port` param
        let config = config_builder
            .add_source(Environment::with_prefix("TODO_").separator("__"))
            .build()?;
        config.try_deserialize()
    }
}

pub const DEFAULT_CONFIG_FILE_NAME: &str = "todo.toml";

#[derive(Debug)]
pub struct ConfigLoader {
    config_file_name: String,
    root_config_file: PathBuf,
    config_file: Option<PathBuf>,
}

impl Default for ConfigLoader {
    fn default() -> Self {
        let root_config_file = home::home_dir()
            .unwrap_or_default()
            .join(".todo")
            .join(DEFAULT_CONFIG_FILE_NAME);

        Self {
            config_file_name: DEFAULT_CONFIG_FILE_NAME.into(),
            root_config_file,
            config_file: None,
        }
    }
}

impl ConfigLoader {
    pub fn with_config_file_name(mut self, config_file_name: impl Into<String>) -> Self {
        self.config_file_name = config_file_name.into();
        self
    }

    pub fn with_root_config_file(mut self, default_root_config_file: impl Into<PathBuf>) -> Self {
        self.root_config_file = default_root_config_file.into();
        self
    }

    pub fn with_config_file(mut self, config_file: impl Into<PathBuf>) -> Self {
        self.config_file = Some(config_file.into());
        self
    }

    pub fn maybe_with_config_file(mut self, config_file: Option<impl Into<PathBuf>>) -> Self {
        self.config_file = config_file.map(Into::into);
        self
    }

    pub fn config_file_name(&self) -> &str {
        &self.config_file_name
    }

    pub fn root_config_file(&self) -> &Path {
        self.root_config_file.as_path()
    }

    pub fn load(self) -> anyhow::Result<ConfigProfile> {
        let Self {
            config_file_name,
            root_config_file,
            config_file,
        } = self;
        let mut config_builder = ConfigBuilder::default();

        let root_config_file = env::var("TODO__ROOT_CONFIG").map(PathBuf::from).or_else(|err| {
            if let VarError::NotPresent = err {
                Ok(root_config_file)
            } else {
                Err(err)
            }
        })?;
        if root_config_file.exists() {
            config_builder.add_path(&root_config_file);
        }

        let config_dir = env::current_dir()?;
        let config_file = config_file
            .or_else(|| find_in_dir_and_parents(&config_dir, &config_file_name))
            .unwrap_or_else(|| config_dir.join(&config_file_name));

        if config_file.exists() {
            config_builder.add_path(&config_file);
        }

        Ok(ConfigProfile {
            config: config_builder.build()?,
            root_config_file,
            config_file,
        })
    }
}

pub struct ConfigProfile {
    pub config: Config,
    pub root_config_file: PathBuf,
    pub config_file: PathBuf,
}
