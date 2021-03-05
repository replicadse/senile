# Reference

## Disclaimer

All features that are marked as `experimental` are _not_ considered a public API and therefore eplicitly not covered by the backwards-compatibility policy inside a major version (see [semver v2](https://semver.org)). Use these features on your own risk!

## TODO format in code

The default format is `// TODO!($priority, $assignee, $context_lines_below): $content`.\
The priority and assignee are self explanatory. You can specify whatever string you want inside them.\
The `$context_lines_below` argument tells the tool how many lines it shall include _below_ your comment as context.

## Application level arguments

|Name|Short|Long|Description|
|-- |-- |-- |-- |
|Experimental|-e|--experimental|Activates experimental features that are not stable yet. All features that are marked as experimental are ignored when keeping backwards compatibility inside one major version.|

## Commands

|Command|Description|Status|
|-- |-- |-- |
|help|Prints the help to `STDOUT`.|stable|
|collect|Collects the todo statements|stable|

## `collect` command flags

|Name|Short|Long|Description|Remark|Status|
|-- |-- |-- |-- |-- |--|
|Path|-p|--path|The path from which to start traversal.|None|stable|
|Workers|-w|--workers|The amount of workers (threads) used when parsing the collected files.|None|stable|
|Filter|-f|--filter|The regex for filtering the files that are to be parsed.|None|stable|
|Format||--format|The format of the todo statements that are parsed. It is the start literal followed by the end literal, separated with a comma.|Example: `--format="// TODO\!(,):"`|stable|

**Examples:**
- Basic usage:\
`senile collect`
- Specifying the root directory:\
`senile collect -p ./src` 
- Specifying the file filter regex (only .rs files):\
`senile collect -f="\.rs$"`
- Specifying the todo statement format (`## TODO~[[min, myself, 0]]:`:\
`senile collect --format="## TODO~[[,]]:"

