#![deny(missing_docs)]

//! senile

use std::error::Error;

mod args;
mod commands;
mod crawler;
mod error;
mod parser;
mod types;

use crate::{
    args::{
        ClapArgumentLoader,
        Command,
    },
    commands::collect,
};

// TODO!(aw,min,1): Wow, it actually works!
fn main() -> Result<(), Box<dyn Error>> {
    let args = ClapArgumentLoader::load()?;
    match args.command {
        | Command::Collect {
            path,
            filter,
            workers,
            literals,
        } => {
            collect(path, filter, workers, literals)?;
            Ok(())
        },
    }
}
