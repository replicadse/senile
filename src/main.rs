use std::error::Error;

use futures::executor::block_on;
use walkdir::WalkDir;
mod args;
mod error;
use std::{
    collections::HashMap,
    fs,
    io::{
        prelude::*,
        Cursor,
    },
};

use args::{
    ClapArgumentLoader,
    Command,
};
use fancy_regex::Regex;
use serde::{
    Deserialize,
    Serialize,
};

use crate::error::ParserError;

// TODO!(2, haw): Add multithreading support for reading the files.


struct ContentParserParams {
    file: String,
    todo_start_str_combined: String
}
struct ContentParserState {
}
struct ContentParser<'s> {
    cursor: &'s mut Cursor::<Vec<u8>>,
    info: ContentParserParams,
    #[allow(dead_code)]
    state: ContentParserState,
}
impl<'s> ContentParser<'s> {
    pub fn new(cursor: &'s mut Cursor::<Vec<u8>>, info: ContentParserParams) -> Self {
        Self {
            cursor,
            info,
            state: ContentParserState {
            }
        }
    }

    pub async fn parse(&mut self) -> Result<Vec<ToDoItem>, Box<dyn Error>> {
        let mut todos = Vec::<ToDoItem>::new();
        let mut buf = String::new();
        let mut line = 0u32;
        loop {
            match self.cursor.read_line(&mut buf) {
                | Ok(size) => {
                    if size <= 0 {
                        break;
                    }
                },
                | Err(_) => break,
            }

            buf = buf.trim().to_owned();
            if let Ok(res) = self.parse_buf(line, &buf).await {
                match res {
                    | Some(v) => {
                        todos.push(v);
                    },
                    | None => {},
                }
            }

            buf.clear();
            line += 1;
        }
        Ok(todos)
    }

    async fn parse_buf(
        &mut self,
        line: u32,
        buf: &str,
    ) -> Result<Option<ToDoItem>, Box<dyn Error>> {
        if let Some(start_idx) = buf.find(&self.info.todo_start_str_combined) {
            let p_error = ParserError::new(&format!("failed to parse line {} in file {}", line, self.info.file));
            let sub_buf = &buf[start_idx..];
            if sub_buf.len() < MIN_TODO_LEN {
                return Err(Box::new(p_error.clone()));
            }
            let p_start_idx = TODO_START_STR.len() + TODO_PARAMS_PARANTHESES_START.len() - 1;
            let p_close_idx = sub_buf
                .find(TODO_PARAMS_PARANTHESES_END)
                .ok_or(Box::new(p_error.clone()))?;
            let parameters = &mut sub_buf[p_start_idx + 1..p_close_idx].split(TODO_PARAMS_SEPARATOR);
            let prio = parameters.next().ok_or(Box::new(p_error.clone()))?.trim();
            let assignee = parameters.next().ok_or(Box::new(p_error.clone()))?.trim();
            let content = sub_buf[p_close_idx + 1..].trim();

            Ok(Some(ToDoItem {
                priority: prio.to_owned(),
                body: content.to_owned(),
                assignee: assignee.to_owned(),
                file: self.info.file.to_owned(),
                line,
            }))
        } else {
            Ok(None)
        }
    }
}


#[derive(Debug, Serialize, Deserialize)]
struct ToDoItem {
    priority: String,
    body: String,
    assignee: String,
    file: String,
    line: u32,
}
const TODO_START_STR: &'static str = "// TODO!";
const TODO_PARAMS_PARANTHESES_START: &'static str = "(";
const TODO_PARAMS_PARANTHESES_END: &'static str = "):";
const TODO_PARAMS_SEPARATOR: &'static str = ",";
const MIN_TODO_LEN: usize =
    TODO_START_STR.len() + TODO_PARAMS_PARANTHESES_START.len() + TODO_PARAMS_PARANTHESES_END.len() + 2;


async fn collect(path: &str, filter: &str) -> Result<(), Box<dyn Error>> {
    let mut all_todos = HashMap::<String, Vec<ToDoItem>>::new();
    let mut todo_start_str_combined = String::new();
    todo_start_str_combined.push_str(TODO_START_STR);
    todo_start_str_combined.push_str(TODO_PARAMS_PARANTHESES_START);
    let matcher = Regex::new(filter)?;
    for entry in WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| !e.file_type().is_dir())
    {
        let entry_path = entry.into_path();
        let entry_path_str = entry_path.to_str().unwrap();
        if !matcher.is_match(entry_path_str)? {
            continue;
        }
        let file_content = fs::read(entry_path_str)?;
        let mut cursor = Cursor::new(file_content);

        let mut parser = ContentParser::new(&mut cursor, ContentParserParams {
            file: entry_path_str.to_owned(),
            todo_start_str_combined: todo_start_str_combined.clone(),
        });

        if let Ok(todos) = parser.parse().await {
            // sort in todos
            for v in todos {
                all_todos.entry(v.priority.to_owned()).or_insert(Vec::new());
                all_todos.get_mut(&v.priority.to_owned()).unwrap().push(v);
            }
        }
    }
    let output = serde_json::to_vec(&all_todos)?;
    let stdout = std::io::stdout();
    let mut stdout_lock = stdout.lock();
    stdout_lock.write(&output)?;
    stdout_lock.flush()?;
    Ok(())
}

async fn main_async() -> Result<(), Box<dyn Error>> {
    let args = ClapArgumentLoader::load().await?;
    match args.command {
        | Command::Collect { path, filter } => {
            collect(&path, &filter).await?;
            Ok(())
        },
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    block_on(main_async())
}
