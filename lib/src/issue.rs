use indexmap::IndexSet;

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
