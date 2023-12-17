use std::env;
use std::env::VarError;
use std::path::{Path, PathBuf};

pub use config::ConfigError;
use config::{Environment, File};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    #[serde(default = "projects_config_file_default")]
    pub projects_config_file: PathBuf,

    #[serde(default = "project_config_file_default")]
    pub project_config_file: PathBuf,

    pub project_search_dirs: Vec<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            projects_config_file: projects_config_file_default(),
            project_config_file: project_config_file_default(),
            project_search_dirs: Default::default(),
        }
    }
}

fn projects_config_file_default() -> PathBuf {
    "projects.toml".into()
}

fn project_config_file_default() -> PathBuf {
    "Project.toml".into()
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

#[derive(Debug)]
pub struct ConfigLoader {
    config_file_name: String,
    root_config_file: PathBuf,
    config_file: Option<PathBuf>,
    project_dir: Option<PathBuf>,
}

impl Default for ConfigLoader {
    fn default() -> Self {
        let default_config_file_name = "todo.toml";
        Self {
            config_file_name: default_config_file_name.into(),
            root_config_file: PathBuf::from("~").join(".todo").join(default_config_file_name),
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

        let mut project_dir = project_dir.map(Ok).unwrap_or_else(|| env::current_dir())?;
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
