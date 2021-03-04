use std::{
    error::Error,
    result::Result,
};

use crate::error::UnknownCommandError;

#[derive(Debug)]
pub struct CallArgs {
    pub privilege: Privilege,
    pub command: Command,
}

impl CallArgs {
    pub async fn validate(&self) -> Result<(), Box<dyn Error>> {
        match self.privilege {
            | Privilege::Normal => Ok(()),
            | Privilege::Experimental => Ok(()),
        }
    }
}

#[derive(Debug)]
pub enum Privilege {
    Normal,
    Experimental,
}

#[derive(Debug)]
pub enum Command {
    Collect {
        path: String,
        filter: String,
        workers: usize,
    },
}

pub struct ClapArgumentLoader {}

impl ClapArgumentLoader {
    pub async fn load() -> Result<CallArgs, Box<dyn Error>> {
        let command = clap::App::new("senile")
            .version(env!("CARGO_PKG_VERSION"))
            .about("senile")
            .author("Weber, Heiko Alexander <haw@voidpointergroup.com>")
            .arg(
                clap::Arg::with_name("experimental")
                    .short("e")
                    .long("experimental")
                    .value_name("EXPERIMENTAL")
                    .help("Enables experimental features that do not count as stable.")
                    .required(false)
                    .takes_value(false),
            )
            .subcommand(
                clap::App::new("collect")
                    .about("Collects the ToDo items from the given folder.")
                    .arg(
                        clap::Arg::with_name("path")
                            .short("p")
                            .long("path")
                            .value_name("PATH")
                            .help("The path on which to start with collecting.")
                            .default_value("./")
                            .multiple(false)
                            .required(false)
                            .takes_value(true))
                    .arg(
                        // ^(?!(\.\/node_modules\/|\.\/target\/)).*(\.rs|\.asm).*$      --> example for including all
                        // .rs and .asm files that are not in [./node_modules/ or ./target/] root folders
                        clap::Arg::with_name("filter")
                            .short("f")
                            .long("filter")
                            .value_name("FILTER")
                            .help("The regex pattern for filtering the files to include. An example that includes only [.rs | .asm] files if their path does not start with [./node_modules/ | ./target/] is the following line: \n^(?!(\\.\\/node_modules\\/|\\.\\/target\\/)).*(\\.rs|\\.asm).*$\n")
                            .default_value("(?s).*") // match anything
                            .multiple(false)
                            .required(false)
                            .takes_value(true))
                    .arg(
                        clap::Arg::with_name("workers")
                            .short("w")
                            .long("workers")
                            .value_name("WORKERS")
                            .help("The amount of worker threads used when parsing the file contents.")
                            .default_value("4")
                            .multiple(false)
                            .required(false)
                            .takes_value(true)))
            .get_matches();

        let privilege = if command.is_present("experimental") {
            Privilege::Experimental
        } else {
            Privilege::Normal
        };

        let cmd = if let Some(x) = command.subcommand_matches("collect") {
            Command::Collect {
                path: x.value_of("path").unwrap().to_owned(), // should be covered by the clap arg being required
                filter: x.value_of("filter").unwrap().to_owned(), // same as above
                workers: x.value_of("workers").unwrap().parse::<usize>()?,
            }
        } else {
            return Err(Box::new(UnknownCommandError::new("unknown command")));
        };

        let callargs = CallArgs {
            privilege,
            command: cmd,
        };

        callargs.validate().await?;
        Ok(callargs)
    }
}
