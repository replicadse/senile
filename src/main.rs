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
