# senile

[![crates.io](https://img.shields.io/crates/v/senile.svg)](https://crates.io/crates/senile)
[![crates.io](https://img.shields.io/crates/d/senile?label=crates.io%20downloads)](https://crates.io/crates/senile)
[![website](https://img.shields.io/badge/AUR-package-blue)](https://aur.archlinux.org/packages/senile/)\
[![pipeline](https://github.com/replicadse/senile/workflows/pipeline/badge.svg)](https://github.com/replicadse/senile/actions?query=workflow%3Apipeline)
[![dependency status](https://deps.rs/repo/github/replicadse/senile/status.svg)](https://deps.rs/repo/github/replicadse/senile)\
[![docs.rs](https://img.shields.io/badge/docs.rs:crate-latest-blue)](https://docs.rs/crate/senile/latest)
[![docs.rs](https://img.shields.io/badge/docs.rs:docs-latest-blue)](https://docs.rs/senile/latest/senile/)
[![website](https://img.shields.io/badge/home-GitHub-blue)](https://github.com/replicadse/senile)
[![website](https://img.shields.io/badge/website-GitHub-blue)](https://replicadse.github.io/senile)
## What is senile?

Your are senile because you can not remember all the ToDos that you have in your code. So in fact you are obviously unable to keep track of them. That is what senile is for.

## Usage

1) `cargo install senile`
2) `senile collect -p="./src" > todos.json`
3) profit

## Jokes aside, how does it work?

`senile` collects all todo statements recursively from the given directory/file (tree). It collects information about the todo body, the priority, the file and the line in that file.\
It will output a json formatted string to `STDOUT` as follows:\
```json
[
  {
    "prio": "$priority",
    "assignee": "$assignee",
    "body": "$todo_body",
    "context": [
      "$context_line",
      "$context_line",
      ...
    ],
    "file": "$fq_relative_file_path",
    "line": $line
  },
  ...
]
Note that these entries are string-sorted by their assigned priority.
