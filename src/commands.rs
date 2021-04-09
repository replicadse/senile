use std::{
    error::Error,
    fs,
    io::{
        prelude::*,
        Cursor,
    },
    thread,
};

use crossbeam::{
    channel::unbounded,
    sync::WaitGroup,
};
use fancy_regex::Regex;
use threadpool::ThreadPool;

use crate::{
    crawler::Crawler,
    parser::{
        ContentParser,
        ContentParserParams,
    },
    types::ToDoItem,
};

#[cfg_attr(doc, aquamarine::aquamarine)]
/// Function to collect files in a tree and parse their content to match todo
/// statements. The internal workings are described in the following diagram:
///
/// ```mermaid
/// graph TD
///     crawler([crawler thread]) --> crawler_c([channel])
///     collector([collector]) --> stdout([stdout])
///     subgraph threadpool[thread pool]
///         thread_0([thread 0])
///         thread_1([thread 1])
///         thread_n([thread n])
///     end
///     thread_0 -. consume .-> crawler_c
///     thread_1 -. consume .-> crawler_c
///     thread_n -. consume .-> crawler_c
///     thread_0 -. publish .-> collector
///     thread_1 -. publish .-> collector
///     thread_n -. publish .-> collector
/// ```
pub fn collect(
    path: String,
    filter: String,
    workers: usize,
    literals: Vec<(String, String)>,
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
        let thread_literals = literals.clone();
        pool.execute(move || {
            let dothis = move || -> Result<(), Box<dyn Error>> {
                let content = fs::read(&s)?;
                let mut cursor = Cursor::new(content);
                let mut parser = ContentParser::new(&mut cursor, ContentParserParams {
                    file: s,
                    literals: thread_literals,
                })
                .unwrap();
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

    let mut all_todos = Vec::<ToDoItem>::new();
    // drop orginal sender_parser to eliminate the +1 original copy from num_workers
    // + 1 (original)
    drop(sender_parser);
    for todo in receiver_parser {
        all_todos.push(todo);
    }
    all_todos.sort_by(|a, b| a.priority.cmp(&b.priority));
    wg.wait();

    // let output = serde_json::to_vec_pretty(&all_todos)?;
    let output = serde_json::to_vec(&all_todos)?;
    let stdout = std::io::stdout();
    let mut stdout_lock = stdout.lock();
    stdout_lock.write(&output)?;
    stdout_lock.flush()?;
    Ok(())
}
