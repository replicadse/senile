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
    collections::BTreeMap,
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
    start_literal: String,
    end_literal: String,
    min_todo_length: usize,
}
struct ContentParserState {}
struct ContentParser<'s> {
    cursor: &'s mut Cursor<Vec<u8>>,
    params: ContentParserParams,
    #[allow(dead_code)]
    state: ContentParserState,
}
impl<'s> ContentParser<'s> {
    pub fn new(cursor: &'s mut Cursor<Vec<u8>>, params: ContentParserParams) -> Self {
        Self {
            cursor,
            params,
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
                | Err(_) => continue,
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

    // |01234567 // TODO!(a, b, c): test
    //  ^        ^        ^      ^
    //  0        9        18     25

    fn parse_buf(&mut self, line: u32, buf: &str) -> Result<Option<ToDoItem>, Box<dyn Error>> {
        if let Some(start_idx) = buf.find(&self.params.start_literal) {
            let p_error = ParserError::new(&format!("failed to parse line {} in file {}", line, self.params.file));
            let sub_buf = &buf[start_idx..];
            if sub_buf.len() < self.params.min_todo_length {
                return Err(Box::new(p_error.clone()));
            }
            let params_start_idx = self.params.start_literal.len(); // index of first char of params
            let params_close_idx = sub_buf[params_start_idx..]
                .find(&self.params.end_literal)
                .ok_or(Box::new(p_error.clone()))?
                + params_start_idx;
            let parameters = &mut sub_buf[params_start_idx..params_close_idx].split(C_TODO_PARAM_SEPARATOR);
            let prio = parameters.next().ok_or(Box::new(p_error.clone()))?.trim();
            let assignee = parameters.next().ok_or(Box::new(p_error.clone()))?.trim();
            let next_lines = parameters.next().ok_or(Box::new(p_error.clone()))?.trim();
            let next_lines_nr = next_lines.parse::<usize>()?;
            let content = sub_buf[params_close_idx + &self.params.end_literal.len()..].trim();
            let mut context = Vec::<String>::new();
            for _ in 0..next_lines_nr {
                let mut line_buf = String::new();
                if let Ok(size) = self.cursor.read_line(&mut line_buf) {
                    if size <= 0 {
                        break;
                    }
                    context.push(line_buf);
                }
            }

            Ok(Some(ToDoItem {
                priority: prio.to_owned(),
                body: content.to_owned(),
                assignee: assignee.to_owned(),
                context,
                file: self.params.file.to_owned(),
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
    context: Vec<String>,
    file: String,
    line: u32,
}

// TODO!(min,haw,2): make these modifiable, maybe?
const C_TODO_PARAM_SEPARATOR: &str = ",";
const C_TODO_PARAM_COUNT: usize = 3;

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

fn collect(
    path: String,
    filter: String,
    workers: usize,
    start_literal: String,
    end_literal: String,
) -> Result<(), Box<dyn Error>> {
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
    let min_todo_length = start_literal.len()
        + C_TODO_PARAM_COUNT
        + (C_TODO_PARAM_SEPARATOR.len() * C_TODO_PARAM_COUNT - 1)
        + end_literal.len();

    for s in receiver_crawler.iter() {
        let thread_wg = wg.clone();
        let thread_sender = sender_parser.clone();
        let thread_start_literal = start_literal.clone();
        let thread_end_literal = end_literal.clone();
        pool.execute(move || {
            let dothis = move || -> Result<(), Box<dyn Error>> {
                let content = fs::read(&s)?;
                let mut cursor = Cursor::new(content);
                let mut parser = ContentParser::new(&mut cursor, ContentParserParams {
                    file: s,
                    start_literal: thread_start_literal,
                    end_literal: thread_end_literal,
                    min_todo_length,
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

    let mut all_todos = BTreeMap::<String, Vec<ToDoItem>>::new();
    drop(sender_parser); // drop orginal sender_parser to eliminate the +1 original copy from
                         // num_threads+1
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
        | Command::Collect {
            path,
            filter,
            workers,
            start_literal,
            end_literal,
        } => {
            collect(path, filter, workers, start_literal, end_literal)?;
            Ok(())
        },
    }
}
