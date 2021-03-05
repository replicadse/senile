use std::error::Error;

use crossbeam::channel::Sender;
use fancy_regex::Regex;
use walkdir::WalkDir;

pub struct Crawler<'s> {
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
