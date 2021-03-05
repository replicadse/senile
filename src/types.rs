use serde::{
    Deserialize,
    Serialize,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct ToDoItem {
    pub priority: String,
    pub body: String,
    pub assignee: String,
    pub context: Vec<String>,
    pub file: String,
    pub line: u32,
}
