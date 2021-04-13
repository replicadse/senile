use std::error::Error;

use crossbeam::channel::Sender;
use fancy_regex::Regex;
use walkdir::WalkDir;

/// The crawler type that goes through a (sub-)tree of a file/directory (inode)
/// and recursively gets files.
pub struct Crawler<'s> {
    matcher: &'s Regex,
}
impl<'s> Crawler<'s> {
    /// Constructor for the crawler.
    /// Accepting the regex which shall match the (fully qualified, relative)
    /// file names.
    pub fn new(matcher: &'s Regex) -> Self {
        Self { matcher }
    }

    /// Executing the crawl, sending the results during the operation through
    /// the sender argument.
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
