use std::{
    error::Error,
    io::{
        BufRead,
        Cursor,
    },
};

use crate::{
    error::ParserError,
    types::ToDoItem,
};

// TODO!(min,haw,2): make these modifiable, maybe?
const C_TODO_PARAM_SEPARATOR: &str = ",";
const C_TODO_PARAM_COUNT: usize = 3;

pub struct ContentParserParams {
    pub file: String,
    pub start_literal: String,
    pub end_literal: String,
}
pub struct ContentParser<'s> {
    cursor: &'s mut Cursor<Vec<u8>>,
    params: ContentParserParams,
    min_todo_length: usize,
}
impl<'s> ContentParser<'s> {
    pub fn new(cursor: &'s mut Cursor<Vec<u8>>, params: ContentParserParams) -> Self {
        let min_todo_length = params.start_literal.len()
            + C_TODO_PARAM_COUNT
            + (C_TODO_PARAM_SEPARATOR.len() * C_TODO_PARAM_COUNT - 1)
            + params.end_literal.len();
        Self {
            cursor,
            params,
            min_todo_length,
        }
    }

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
        if let Some(start_idx) = buf.find(&self.params.start_literal) {
            let p_error = ParserError::new(&format!("failed to parse line {} in file {}", line, self.params.file));
            let sub_buf = &buf[start_idx..];
            if sub_buf.len() < self.min_todo_length {
                return Err(Box::new(p_error.clone()));
            }
            let params_start_idx = self.params.start_literal.len(); // index of first char of params
            let params_close_idx = sub_buf[params_start_idx..]
                .find(&self.params.end_literal)
                .ok_or(Box::new(p_error.clone()))?
                + params_start_idx;
            let parameters = &mut sub_buf[params_start_idx..params_close_idx].split(C_TODO_PARAM_SEPARATOR);
            let prio = parameters.next().ok_or(Box::new(p_error.clone()))?.trim();
            let assignee = parameters.next().ok_or(Box::new(p_error.clone()))?.trim();
            let next_lines = parameters.next().ok_or(Box::new(p_error.clone()))?.trim();
            let next_lines_nr = next_lines.parse::<usize>()?;
            let content = sub_buf[params_close_idx + &self.params.end_literal.len()..].trim();
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

            Ok(Some(ToDoItem {
                priority: prio.to_owned(),
                body: content.to_owned(),
                assignee: assignee.to_owned(),
                context,
                file: self.params.file.to_owned(),
                line,
            }))
        } else {
            Ok(None)
        }
    }
}
