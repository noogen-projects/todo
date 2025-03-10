use std::collections::HashMap;
use std::hash::Hash;
use std::io;
use std::path::{Path, PathBuf};

use indexmap::{IndexMap, IndexSet};
use regex::Regex;
use todo_lib::plan::Plan;
use todo_lib::project::Project;
use walkdir::{DirEntry, WalkDir};

use crate::config::ProjectConfig;
use crate::generator::IntIdGenerator;
use crate::plan::LoadProjectPlan;
use crate::project::LoadProject;
use crate::Placement;

pub struct FsTracker<PID = String, ID = u64> {
    projects: IndexMap<PID, Project<PID>>,
    project_root_dirs: HashMap<PID, PathBuf>,
    parents: IndexMap<PID, PID>,
    planes: HashMap<PID, Plan<ID>>,
}

impl<PID: Clone + Hash + Eq> FsTracker<PID> {
    pub fn new(
        project_configs: IndexMap<PID, ProjectConfig<PID>>,
        manifest_filename_regex: &Regex,
        todo_filename_regex: &Regex,
    ) -> io::Result<Self> {
        let mut projects = IndexMap::new();
        let mut project_root_dirs = HashMap::new();
        let mut parents = IndexMap::new();
        let mut planes = HashMap::new();

        for (parent_id, config) in &project_configs {
            parents.extend(config.subprojects.iter().cloned().map(|id| (id, parent_id.clone())));
        }

        for (id, config) in project_configs {
            if let Some(project_root) = config.root_dir.clone() {
                if let Some(plan) = load_project_plan(&config, manifest_filename_regex, todo_filename_regex)? {
                    planes.insert(id.clone(), plan);
                }
                project_root_dirs.insert(id.clone(), project_root);
            }
            let project = Project::load(id.clone(), parents.get(&id).cloned(), config);
            projects.insert(id, project);
        }

        Ok(Self {
            projects,
            project_root_dirs,
            parents,
            planes,
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

    pub fn project_issues(&self, id: &PID) -> Option<&Plan<u64>> {
        self.planes.get(id)
    }

    pub fn project_path(&self, id: &PID) -> Option<&Path> {
        self.project_root_dirs.get(id).map(|path| path.as_path())
    }
}

pub fn find_match_files<'a>(root_dir: &Path, regex: &'a Regex) -> impl Iterator<Item = DirEntry> + 'a {
    WalkDir::new(root_dir)
        .max_depth(1)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            let file_name = entry.file_name().to_string_lossy();
            regex.is_match(file_name.as_ref())
        })
}

fn load_project_plan<PID>(
    project_config: &ProjectConfig<PID>,
    manifest_filename_regex: &Regex,
    todo_filename_regex: &Regex,
) -> io::Result<Option<Plan<u64>>>
where
    PID: Hash + Eq,
{
    let Some(project_root) = project_config.root_dir.clone() else {
        return Ok(None);
    };
    let id_generator = IntIdGenerator::new(project_config.start_id.unwrap_or(1));
    let mut plan = Plan::new();
    let mut plan_exists = false;

    let manifest_file_path = find_match_files(&project_root, manifest_filename_regex)
        .next()
        .map(|entry| entry.into_path());
    let todo_file_path = find_match_files(&project_root, todo_filename_regex)
        .next()
        .map(|entry| entry.into_path());

    if let Some(manifest_file_path) = manifest_file_path {
        let source = Placement::CodeBlockInFile(manifest_file_path);
        plan = match Plan::load(&source, &id_generator)? {
            Some(plan) => plan,
            None => return Ok(None),
        };
        plan_exists = true;
    }

    if let Some(todo_file_path) = todo_file_path {
        let source = Placement::WholeFile(todo_file_path);
        plan = plan.merge(match Plan::load(&source, &id_generator)? {
            Some(plan) => plan,
            None => return Ok(None),
        });
        plan_exists = true;
    }

    if plan_exists {
        Ok(Some(plan))
    } else {
        Ok(None)
    }
}
