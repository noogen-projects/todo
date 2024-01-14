use indexmap::{IndexMap, IndexSet};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum LinkType {
    FinishToStart,
    StartToStart,
    FinishToFinish,
    StartToFinish,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum DependencyType {
    Before,
    After,
    Blocks,
    IsBlockedBy,
    Contains,
    IsContainedIn,
    RelatesTo,
    AssociatedWith,
    Other(String),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Relation {
    pub link: LinkType,
    pub dependency: DependencyType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IssueRelation<ID> {
    pub to_id: ID,
    pub relation: Relation,
}

#[derive(Debug, Clone)]
pub struct Issue<ID> {
    pub id: ID,
    pub parent_id: Option<ID>,
    pub name: String,
    pub content: String,
    pub subissues: IndexSet<ID>,
    pub relations: Vec<IssueRelation<ID>>,
}

impl<ID> Issue<ID> {
    pub fn new(id: ID, name: impl Into<String>) -> Self {
        Self {
            id,
            parent_id: None,
            name: name.into(),
            content: Default::default(),
            subissues: Default::default(),
            relations: Default::default(),
        }
    }

    pub fn with_parent_id(mut self, parent_id: ID) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    pub fn with_content(mut self, content: impl Into<String>) -> Self {
        self.content = content.into();
        self
    }
}

pub struct Milestone<ID> {
    pub id: ID,
    pub name: String,
    pub needed_issues: IndexSet<ID>,
}

impl<ID> Milestone<ID> {
    pub fn new(id: ID, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            needed_issues: Default::default(),
        }
    }
}

#[derive(Eq, Hash, PartialEq)]
pub enum Step<ID> {
    Issue(ID),
    Milestone(ID),
}

#[derive(Default)]
pub struct Plan<ID> {
    pub steps: IndexSet<Step<ID>>,
}

#[derive(Default)]
pub struct PlannedIssues<ID> {
    issues: IndexMap<ID, Issue<ID>>,
    milestones: IndexMap<ID, Milestone<ID>>,
    plan: Plan<ID>,
}

impl<ID> PlannedIssues<ID> {
    pub fn new() -> Self {
        Self {
            issues: IndexMap::new(),
            milestones: IndexMap::new(),
            plan: Plan { steps: IndexSet::new() },
        }
    }
}

impl<ID: std::hash::Hash + Eq + PartialEq + Clone> PlannedIssues<ID> {
    pub fn get_issue(&self, id: &ID) -> Option<&Issue<ID>> {
        self.issues.get(id)
    }

    pub fn get_milestone(&self, id: &ID) -> Option<&Milestone<ID>> {
        self.milestones.get(id)
    }

    pub fn steps(&self) -> impl IntoIterator<Item = &Step<ID>> {
        &self.plan.steps
    }

    pub fn add_issue(&mut self, issue: Issue<ID>) {
        self.plan.steps.insert(Step::Issue(issue.id.clone()));
        self.issues.insert(issue.id.clone(), issue);
    }

    pub fn add_milestone(&mut self, milestone: Milestone<ID>) {
        self.plan.steps.insert(Step::Milestone(milestone.id.clone()));
        self.milestones.insert(milestone.id.clone(), milestone);
    }

    pub fn add_issues(&mut self, issues: impl IntoIterator<Item = Issue<ID>>) {
        for issue in issues {
            self.add_issue(issue);
        }
    }
}
