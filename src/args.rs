use std::{
    error::Error,
    result::Result,
};

use crate::error::{
    ArgumentError,
    UnknownCommandError,
};

#[derive(Debug)]
/// Combined arguments struct for the invoked command incl. all necessary
/// information.
pub struct CallArgs {
    /// The privilege with which the program was called.
    pub privilege: Privilege,
    /// The subcommand that was called incl. all arguments and parameters.
    pub command: Command,
}

impl CallArgs {
    /// Validating the arguments since some commands may only be called with
    /// certain privileges, arguments being XOR or similar.
    pub fn validate(&self) -> Result<(), Box<dyn Error>> {
        match self.privilege {
            | Privilege::Normal => Ok(()),
            | Privilege::Experimental => Ok(()),
        }
    }
}

#[derive(Debug)]
/// The privilege.
pub enum Privilege {
    /// Normal privileges identify the normal scenario.
    Normal,
    /// Experimental privileges give access to unstable features.
    Experimental,
}

#[derive(Debug)]
/// The (sub-)command representation for the call args.
pub enum Command {
    /// Collect subcommand representation.
    Collect {
        /// The path from which to crawl.
        path: String,
        /// The (regex) filter on files in the (sub-)tree below incl. `path`.
        filter: String,
        /// The amount of worker(-thread)s.
        workers: usize,
        /// Tuples of start&end literals
        literals: Vec<(String, String)>,
    },
}

/// The type that parses the arguments to the program.
pub struct ClapArgumentLoader {}

impl ClapArgumentLoader {
    /// Parsing the program arguments with the `clap` trait.
    pub fn load() -> Result<CallArgs, Box<dyn Error>> {
        let command = clap::App::new("senile")
            .version(env!("CARGO_PKG_VERSION"))
            .about("senile")
            .author("Weber, Alexander <aw@voidpointergroup.com>")
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
                        // ^(?!(\.\/node_modules\/|\.\/target\/)).*(\.rs|\.asm)$      --> example for including all
                        // .rs and .asm files that are not in [./node_modules/ or ./target/] root folders
                        clap::Arg::with_name("filter")
                            .short("f")
                            .long("filter")
                            .value_name("FILTER")
                            .help("The regex pattern for filtering the files to include. An example that includes only [.rs | .asm] files if their path does not start with [./node_modules/ | ./target/] is the following line: \n^(?!(\\.\\/node_modules\\/|\\.\\/target\\/)).*(\\.rs|\\.asm)$\n")
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
                            .takes_value(true))
                    .arg(
                        clap::Arg::with_name("format")
                            .long("format")
                            .value_name("FORMAT")
                            .help("The format of the TODO statements in the source. Format separated with commas in the order \"start_literal\", \"end_literal\". To allow more than one format, use this arg multiple times.")
                            .default_value("// TODO!(,):")
                            .multiple(true)
                            .required(false)
                            .takes_value(true)))
            .get_matches();

        let privilege = if command.is_present("experimental") {
            Privilege::Experimental
        } else {
            Privilege::Normal
        };

        let cmd = if let Some(x) = command.subcommand_matches("collect") {
            let formats = x.values_of("format").unwrap();
            let mut literals = Vec::<(String, String)>::new();
            for f in formats {
                let format_arg = f.split(",").collect::<Vec<&str>>();
                if format_arg.len() != 2 {
                    return Err(Box::new(ArgumentError::new("invalid format")));
                };
                literals.push((format_arg[0].to_owned(), format_arg[1].to_owned()))
            }
            Command::Collect {
                path: x.value_of("path").unwrap().to_owned(), // should be covered by the clap arg being required
                filter: x.value_of("filter").unwrap().to_owned(), // same as above
                workers: x.value_of("workers").unwrap().parse::<usize>()?, // same as above
                literals,
            }
        } else {
            return Err(Box::new(UnknownCommandError::new("unknown command")));
        };

        let callargs = CallArgs {
            privilege,
            command: cmd,
        };

        callargs.validate()?;
        Ok(callargs)
    }
}
