#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TaskStatus {
    Initializing,
    Running,
    Finished,
    Failed(String),
}
