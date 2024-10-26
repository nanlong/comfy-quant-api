#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TaskStatus<T> {
    Initializing,
    Running(T),
    Finished,
    Failed(String),
}
