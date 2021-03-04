use std::{
    error::Error,
    str,
    thread,
};

use crossbeam::{
    channel::{
        unbounded,
        Sender,
    },
    sync::WaitGroup,
};
use threadpool::ThreadPool;
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

struct ContentParserParams {
    file: String,
    todo_start_str_combined: String,
}
struct ContentParserState {}
struct ContentParser<'s> {
    cursor: &'s mut Cursor<Vec<u8>>,
    info: ContentParserParams,
    #[allow(dead_code)]
    state: ContentParserState,
}
impl<'s> ContentParser<'s> {
    pub fn new(cursor: &'s mut Cursor<Vec<u8>>, info: ContentParserParams) -> Self {
        Self {
            cursor,
            info,
            state: ContentParserState {},
        }
    }

    pub fn parse(&mut self) -> Result<Vec<ToDoItem>, Box<dyn Error>> {
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
            if let Ok(res) = self.parse_buf(line, &buf) {
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

    fn parse_buf(&mut self, line: u32, buf: &str) -> Result<Option<ToDoItem>, Box<dyn Error>> {
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

struct Crawler<'s> {
    matcher: &'s Regex,
}
impl<'s> Crawler<'s> {
    pub fn new(matcher: &'s Regex) -> Self {
        Self { matcher }
    }

    pub fn crawl(&self, path: &str, sender: Sender<String>) -> Result<(), Box<dyn Error>> {
        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| !e.file_type().is_dir())
        {
            let entry_path = entry.into_path();
            let entry_path_str = entry_path.to_str().unwrap();
            if !self.matcher.is_match(entry_path_str)? {
                continue;
            }
            sender.send(entry_path_str.to_owned())?;
        }
        Ok(())
    }
}

fn collect(path: String, filter: String, workers: usize) -> Result<(), Box<dyn Error>> {
    let (sender_crawler, receiver_crawler) = unbounded();

    // fire and forget thread
    let crawler_thread = thread::spawn(move || {
        let matcher = Regex::new(&filter).unwrap();
        let crawler = Crawler::new(&matcher);
        crawler.crawl(&path, sender_crawler).unwrap();
    });

    let wg = WaitGroup::new();
    let (sender_parser, receiver_parser) = unbounded();

    let pool = ThreadPool::new(workers);

    for s in receiver_crawler.iter() {
        let thread_wg = wg.clone();
        let thread_sender = sender_parser.clone();
        pool.execute(move || {
            let dothis = move || -> Result<(), Box<dyn Error>> {
                // TODO!(1, haw): Move this into shared content parser arguments
                let mut todo_start_str_combined = String::new();
                todo_start_str_combined.push_str(TODO_START_STR);
                todo_start_str_combined.push_str(TODO_PARAMS_PARANTHESES_START);
                let content = fs::read(&s)?;
                let mut cursor = Cursor::new(content);
                let mut parser = ContentParser::new(&mut cursor, ContentParserParams {
                    file: s,
                    todo_start_str_combined,
                });
                let todos = parser.parse()?;
                for t in todos {
                    thread_sender.send(t)?;
                }
                Ok(())
            };
            dothis().unwrap_or_default();
            drop(thread_wg);
        });
    }
    crawler_thread.join().expect("the crawler thread has panicked");

    let mut all_todos = HashMap::<String, Vec<ToDoItem>>::new();
    drop(sender_parser);
    for todo in receiver_parser {
        all_todos.entry(todo.priority.to_owned()).or_insert(Vec::new());
        all_todos.get_mut(&todo.priority).unwrap().push(todo);
    }
    wg.wait();

    // let output = serde_json::to_vec_pretty(&all_todos)?;
    let output = serde_json::to_vec(&all_todos)?;
    let stdout = std::io::stdout();
    let mut stdout_lock = stdout.lock();
    stdout_lock.write(&output)?;
    stdout_lock.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = ClapArgumentLoader::load()?;
    match args.command {
        | Command::Collect { path, filter, workers } => {
            collect(path, filter, workers)?;
            Ok(())
        },
    }
}
