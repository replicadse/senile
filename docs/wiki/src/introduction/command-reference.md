# Command reference

## Disclaimer

All features that are marked as `experimental` are _not_ considered a public API and therefore eplicitly not covered by the backwards-compatibility policy inside a major version (see [semver v2](https://semver.org)). Use these features on your own risk!

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

**Examples:**
* `senile collect`
* `senile collect -p ./src`
* `senile collect -p ./src > todos.json`
