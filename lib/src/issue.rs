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

#[derive(Debug, Clone, PartialEq)]
pub struct Issue<PID, ID> {
    id: ID,
    project_id: PID,
    relations: Vec<IssueRelation<ID>>,
    title: String,
    content: String,
}
