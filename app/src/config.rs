use std::env;
use std::env::VarError;
use std::path::{Path, PathBuf};

pub use config::ConfigError;
use config::{Environment, File};
use indexmap::{IndexMap, IndexSet};
use regex::Regex;
use serde::{Deserialize, Serialize};
use todo_tracker_fs::config::{LoadConfigError, ProjectConfig as TrackerProjectConfig};

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
pub struct ProjectConfig {
    pub path: Option<PathBuf>,

    pub subprojects: IndexSet<String>,
}

impl ProjectConfig {
    pub fn load_tracker_project_config(
        &self,
        project_id: impl Into<String>,
        project_config_file_name: impl AsRef<Path>,
        projects_root_dir: Option<impl Into<PathBuf>>,
    ) -> Option<Result<TrackerProjectConfig, LoadConfigError>> {
        let mut project_path = self.path.clone()?;

        if let Some(projects_root_dir) = projects_root_dir {
            if project_path.is_relative() {
                project_path = projects_root_dir.into().join(project_path)
            }
        }

        let project_config_file_path = if project_path.is_dir() {
            project_path.join(project_config_file_name)
        } else {
            project_path.clone()
        };

        let mut config = match TrackerProjectConfig::load(project_config_file_path) {
            Ok(config) => config,
            Err(err) => {
                if project_path.is_dir() {
                    TrackerProjectConfig::new(project_id.into())
                } else {
                    return Some(Err(err));
                }
            },
        };

        config.path = Some(project_path);
        if !self.subprojects.is_empty() {
            config.subprojects.clone_from(&self.subprojects);
        }

        Some(Ok(config))
    }
}

#[derive(Copy, Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Title {
    #[default]
    Id,
    Name,
    IdName,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct DisplayProjectConfig {
    pub title: Title,
    pub steps: usize,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct DisplayConfig {
    pub project: DisplayProjectConfig,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct SourceConfig {
    #[serde(default = "SourceConfig::default_manifest_filename_regex", with = "serde_regex")]
    pub manifest_filename_regex: Regex,

    #[serde(default = "SourceConfig::default_manifest_filename_example")]
    pub manifest_filename_example: String,

    #[serde(default = "SourceConfig::default_todo_filename_regex", with = "serde_regex")]
    pub todo_filename_regex: Regex,

    #[serde(default = "SourceConfig::default_project_config_file")]
    pub project_config_file: PathBuf,

    pub projects_root_dir: Option<PathBuf>,
}

impl Default for SourceConfig {
    fn default() -> Self {
        Self {
            manifest_filename_regex: Self::default_manifest_filename_regex(),
            manifest_filename_example: Self::default_manifest_filename_example(),
            todo_filename_regex: Self::default_todo_filename_regex(),
            project_config_file: Self::default_project_config_file(),
            projects_root_dir: None,
        }
    }
}

impl SourceConfig {
    pub fn default_manifest_filename_regex() -> Regex {
        Regex::new("(.*)\\.manifest\\.md$").expect("regex mus be correct")
    }

    pub fn default_manifest_filename_example() -> String {
        "test.manifest.md".into()
    }

    pub fn default_todo_filename_regex() -> Regex {
        Regex::new("^TODO\\.md$").expect("regex mus be correct")
    }

    pub fn default_project_config_file() -> PathBuf {
        "Project.toml".into()
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    pub source: SourceConfig,

    pub display: DisplayConfig,

    pub search: SearchConfig,

    pub list: ListConfig,

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
    project_dir: Option<PathBuf>,
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
            project_dir: None,
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

    pub fn with_project_dir(mut self, project_dir: impl Into<PathBuf>) -> Self {
        self.project_dir = Some(project_dir.into());
        self
    }

    pub fn maybe_with_project_dir(mut self, project_dir: Option<impl Into<PathBuf>>) -> Self {
        self.project_dir = project_dir.map(Into::into);
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
            project_dir,
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

        let mut project_dir = project_dir.map(Ok).unwrap_or_else(env::current_dir)?;
        let mut config_file = config_file.unwrap_or_else(|| project_dir.join(&config_file_name));

        if !config_file.exists() {
            let mut current_dir = project_dir.clone();
            while let Some(next_dir) = current_dir.parent().map(ToOwned::to_owned) {
                let next_config_file: PathBuf = next_dir.join(&config_file_name);
                if next_config_file.exists() {
                    project_dir = next_dir;
                    config_file = next_config_file;
                    break;
                } else {
                    current_dir = next_dir;
                }
            }
        }

        if config_file.exists() {
            config_builder.add_path(&config_file);
        }

        Ok(ConfigProfile {
            config: config_builder.build()?,
            root_config_file,
            config_file,
            project_dir,
        })
    }
}

pub struct ConfigProfile {
    pub config: Config,
    pub root_config_file: PathBuf,
    pub config_file: PathBuf,
    pub project_dir: PathBuf,
}
