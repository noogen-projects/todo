use std::hash::Hash;

use todo_lib::project::Project;

use crate::config::ProjectConfig;

pub fn load_project<ID: Hash + Eq>(id: ID, parent: Option<ID>, config: ProjectConfig<ID>) -> Project<ID> {
    let mut project = Project::new(id, config.name.unwrap_or_default()).with_subprojects(config.projects);
    if let Some(parent_id) = parent {
        project.set_parent(parent_id);
    }
    project
}
