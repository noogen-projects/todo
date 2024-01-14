use std::collections::HashMap;
use std::hash::Hash;
use std::io;
use std::path::{Path, PathBuf};

use indexmap::{IndexMap, IndexSet};
use todo_lib::issue::PlannedIssues;
use todo_lib::project::Project;

use crate::config::ProjectConfig;
use crate::generator::IntIdGenerator;
use crate::load;

pub struct FsTracker<PID = String, ID = u64> {
    projects: IndexMap<PID, Project<PID>>,
    paths: HashMap<PID, PathBuf>,
    parents: IndexMap<PID, PID>,
    issues: HashMap<PID, PlannedIssues<ID>>,
}

impl<PID: Clone + Hash + Eq> FsTracker<PID> {
    pub fn new(project_configs: IndexMap<PID, ProjectConfig<PID>>) -> io::Result<Self> {
        let mut projects = IndexMap::new();
        let mut paths = HashMap::new();
        let mut parents = IndexMap::new();
        let mut issues = HashMap::new();

        for (parent_id, config) in &project_configs {
            parents.extend(config.projects.iter().cloned().map(|id| (id, parent_id.clone())));
        }

        for (id, config) in project_configs {
            if let Some(path) = config.path.clone() {
                let id_generator = IntIdGenerator::new(config.start_id.unwrap_or(1));
                if let Some(project_issues) = load::project_planned_issues(&path, &id_generator) {
                    issues.insert(id.clone(), project_issues?);
                }
                paths.insert(id.clone(), path);
            }
            let project = load::project(id.clone(), parents.get(&id).cloned(), config);
            projects.insert(id, project);
        }

        Ok(Self {
            projects,
            paths,
            parents,
            issues,
        })
    }

    pub fn projects(&self) -> &IndexMap<PID, Project<PID>> {
        &self.projects
    }

    pub fn project_parents(&self) -> &IndexMap<PID, PID> {
        &self.parents
    }

    pub fn subprojects(&self) -> IndexMap<PID, IndexSet<PID>> {
        let mut subprojects: IndexMap<_, IndexSet<_>> = IndexMap::new();
        for (child, parent) in &self.parents {
            let child = child.clone();
            if let Some(children) = subprojects.get_mut(parent) {
                children.insert(child);
            } else {
                subprojects.insert(parent.clone(), IndexSet::from([child.clone()]));
            }
        }
        subprojects
    }

    pub fn project_issues(&self, id: &PID) -> Option<&PlannedIssues<u64>> {
        self.issues.get(id)
    }

    pub fn project_path(&self, id: &PID) -> Option<&Path> {
        self.paths.get(id).map(|path| path.as_path())
    }
}
