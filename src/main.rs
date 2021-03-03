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
use serde::{
    Deserialize,
    Serialize,
};

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

async fn collect(path: &str) -> Result<(), Box<dyn Error>> {
    let mut todos = HashMap::<String, Vec<ToDoItem>>::new();
    let mut todo_start_str_combined = String::new();
    todo_start_str_combined.push_str(TODO_START_STR);
    todo_start_str_combined.push_str(TODO_PARAMS_PARANTHESES_START);
    for entry in WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| !e.file_type().is_dir())
    {
        // println!("{:?}", entry.into_path());
        let entry_path = entry.into_path();
        let entry_path_str = entry_path.to_str().unwrap();
        let file_content = fs::read(entry_path_str)?;
        let mut cursor = Cursor::new(file_content);
        let mut buf = String::new();
        let mut line = 0;
        loop {
            match cursor.read_line(&mut buf) {
                | Ok(size) => {
                    if size <= 0 {
                        break;
                    }
                },
                | Err(_) => break,
            }

            buf = buf.trim().to_owned();
            if let Some(start_idx) = buf.find(&todo_start_str_combined) {
                let sub_buf = &buf[start_idx..];
                println!("{}", &sub_buf);
                if sub_buf.len() < MIN_TODO_LEN {
                    continue;
                }
                let p_start_idx = TODO_START_STR.len() + TODO_PARAMS_PARANTHESES_START.len() - 1;
                let p_close_idx = sub_buf.find(TODO_PARAMS_PARANTHESES_END).unwrap();
                let parameters = &mut sub_buf[p_start_idx + 1..p_close_idx].split(TODO_PARAMS_SEPARATOR);
                let prio = parameters.next().unwrap().trim();
                let assignee = parameters.next().unwrap().trim();
                let content = sub_buf[p_close_idx + 1..].trim();
                todos.entry(prio.to_owned()).or_insert(Vec::new());
                todos.get_mut(&prio.to_owned()).unwrap().push(ToDoItem {
                    priority: prio.to_owned(),
                    body: content.to_owned(),
                    assignee: assignee.to_owned(),
                    file: entry_path_str.to_owned(),
                    line,
                });
            }
            buf.clear();
            line += 1;
        }
    }
    let output = serde_json::to_vec(&todos)?;
    let mut stdout = std::io::stdout();
    stdout.write(&output)?;
    stdout.flush()?;
    Ok(())
}

async fn main_async() -> Result<(), Box<dyn Error>> {
    let args = ClapArgumentLoader::load().await?;
    match args.command {
        | Command::Collect { path } => {
            collect(&path).await?;
            Ok(())
        },
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    block_on(main_async())
}
