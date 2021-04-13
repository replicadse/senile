use serde::{
    Deserialize,
    Serialize,
};

/// The typed todo item.
#[derive(Debug, Serialize, Deserialize)]
pub struct ToDoItem {
    /// The free-text priority. Can be something like ['min', 'max', '0', '1']
    /// or whatever.
    pub priority: String,
    /// Main body of the item. This is intended to contain the majority of the
    /// free text.
    pub body: String,
    /// Who this todo belongs to.
    pub assignee: String,
    /// Some context for the item (lines below it's declaration).
    pub context: Vec<String>,
    /// The (relative) file path where it has been collected.
    pub file: String,
    /// Line in the file at which the declaration stood.
    pub line: u32,
}
