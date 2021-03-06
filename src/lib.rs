/// senile
///
/// MIT licensed
///
/// senile is a program for detecting parameterized literals in files. It is not
/// incredibly fast because it has not been optimized for speed but for features
/// (at least in the state it is in right now). Nevertheless, it has quite good
/// performance when going through hundreds or thousands of files due to it's
/// parallelized file crawling / parallel parsing capability.
///
/// The OG use case was simply for myself as I like to write TODO statements
/// right into the source code or other resources. This can lead to me
/// forgetting that I put them there and they vanish into the infinite hunting
/// grounds. I COULD have used `ripgrep` for such thing but I like to
/// parameterize the todos a bit more (incl. assignee, priority and such) and I
/// really wanted a nice-to-parse output format like JSON which is used right
/// now as format for the `STDOUT` output.
///
/// For more information on pretty much everything, consider checking the wiki for it (https://replicadse.github.io/senile/).
pub mod args;
pub mod commands;
pub mod crawler;
pub mod error;
pub mod parser;
pub mod types;
