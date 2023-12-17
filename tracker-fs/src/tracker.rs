use std::collections::HashMap;
use std::hash::Hash;
use std::path::{Path, PathBuf};

use todo_lib::project::Project;

use crate::config::ProjectsConfig;
use crate::project::load_project;

pub struct FsTracker<PID = String> {
    projects: HashMap<PID, Project<PID>>,
    paths: HashMap<PID, PathBuf>,
    parents: HashMap<PID, PID>,
}

impl<PID: Clone + Hash + Eq> FsTracker<PID> {
    pub fn new(projects_config: ProjectsConfig<PID>) -> Self {
        let mut projects = HashMap::new();
        let mut paths = HashMap::new();
        let mut parents = HashMap::new();

        for (parent_id, config) in &projects_config.projects {
            parents.extend(config.projects.iter().cloned().map(|id| (id, parent_id.clone())));
        }

        for (id, config) in projects_config.projects {
            if let Some(path) = config.path.clone() {
                paths.insert(id.clone(), path);
            }
            let project = load_project(id.clone(), parents.get(&id).cloned(), config);
            projects.insert(id, project);
        }

        Self {
            projects,
            paths,
            parents,
        }
    }

    pub fn projects(&self) -> &HashMap<PID, Project<PID>> {
        &self.projects
    }

    pub fn path(&self, id: &PID) -> Option<&Path> {
        self.paths.get(id).map(|path| path.as_path())
    }
}
