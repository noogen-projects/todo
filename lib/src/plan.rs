pub enum Step<ID> {
    Issue(ID),
    Milestone(ID),
}

pub struct Plan {
    steps: IndexSet<Step<ID>>,
}
