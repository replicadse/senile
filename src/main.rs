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

use crate::{
    parser::{
        ContentParser,
        ContentParserParams,
    },
    types::ToDoItem,
};
mod args;
mod error;
mod parser;
mod types;
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
