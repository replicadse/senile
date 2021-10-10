use std::{
    error::Error,
    io::{
        BufRead,
        Cursor,
    },
};

use fancy_regex::Regex;

use crate::{
    error::ParserError,
    types::ToDoItem,
};

const C_TODO_PARAM_SEPARATOR: &str = ",";
const C_TODO_PARAM_COUNT: usize = 3;

/// Parameters for the parser.
pub struct ContentParserParams {
    /// The file (path) for todo item context.
    pub file: String,
    /// The literals for finding the tokens.
    pub literals: Vec<(String, String)>,
}
/// The parser.
pub struct ContentParser<'s> {
    cursor: &'s mut Cursor<Vec<u8>>,
    params: ContentParserParams,
    exprs: Regex,
}
impl<'s> ContentParser<'s> {
    /// Creates a new parser struct with the given parameters.
    pub fn new(cursor: &'s mut Cursor<Vec<u8>>, params: ContentParserParams) -> Result<Self, Box<dyn Error>> {
        let params_regex_str = &format!(".+{}", C_TODO_PARAM_SEPARATOR)
            .repeat(C_TODO_PARAM_COUNT)
            .trim_end_matches(C_TODO_PARAM_SEPARATOR)
            .to_string();
        let mut regex_parts = String::new();
        for lit in &params.literals {
            regex_parts.push_str("(");
            let mut part = String::new();
            part.push_str(&lit.0);
            part.push_str(params_regex_str);
            part.push_str(&lit.1);
            part.push_str(".*");
            regex_parts.push_str(&part);
            regex_parts.push_str(")|");
        }
        regex_parts = regex_parts.trim_end_matches("|").to_owned();
        let exprs = Regex::new(&regex_parts)?;
        Ok(Self { cursor, params, exprs })
    }

    /// Begins parsing the content.
    /// Returns a result of a vector of found ToDo items.
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
        if let Some(caps) = self.exprs.captures(buf)? {
            for cap in caps.iter() {
                if let Some(cap_some) = cap {
                    let substr = &buf[cap_some.start()..cap_some.end()];
                    match self.parse_stmt(line, substr) {
                        | Ok(v) => return Ok(Some(v)),
                        | Err(e) => return Err(e),
                    }
                }
            }
        }
        Ok(None)
    }

    fn parse_stmt(&mut self, line: u32, buf: &str) -> Result<ToDoItem, Box<dyn Error>> {
        let p_error = ParserError::new(&format!("failed to parse line {} in file {}", line, self.params.file));
        for match_pair in &self.params.literals {
            if let Some(start_idx) = buf.find(&match_pair.0) {
                let min_todo_length = match_pair.0.len()
                    + C_TODO_PARAM_COUNT
                    + (C_TODO_PARAM_SEPARATOR.len() * C_TODO_PARAM_COUNT - 1)
                    + match_pair.1.len();
                let sub_buf = &buf[start_idx..];
                if sub_buf.len() < min_todo_length {
                    return Err(Box::new(p_error.clone()));
                }
                let params_start_idx = match_pair.0.len(); // index of first char of params
                let params_close_idx = sub_buf[params_start_idx..]
                    .find(&match_pair.1)
                    .ok_or(Box::new(p_error.clone()))?
                    + params_start_idx;
                let parameters = &mut sub_buf[params_start_idx..params_close_idx].split(C_TODO_PARAM_SEPARATOR);
                let prio = parameters.next().ok_or(Box::new(p_error.clone()))?.trim();
                let assignee = parameters.next().ok_or(Box::new(p_error.clone()))?.trim();
                let next_lines = parameters.next().ok_or(Box::new(p_error.clone()))?.trim();
                let next_lines_nr = next_lines.parse::<usize>()?;
                let content = sub_buf[params_close_idx + &match_pair.1.len()..].trim();
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

                return Ok(ToDoItem {
                    priority: prio.to_owned(),
                    body: content.to_owned(),
                    assignee: assignee.to_owned(),
                    context,
                    file: self.params.file.to_owned(),
                    line,
                });
            }
        }
        Err(Box::new(p_error.clone()))
    }
}
