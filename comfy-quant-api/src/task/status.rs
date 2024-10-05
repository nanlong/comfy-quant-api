#[allow(unused)]
#[derive(Clone, Debug)]
pub enum TaskStatus {
    Initializing,
    Running,
    Finished,
    Failed(String),
}
