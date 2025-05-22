JSON Lines Viewer
---
A terminal-UI to browse through JSON-line files.
The main use case is to support the analysis of comprehensive application logs in 'Json line' format.


## Install

### Developer way (compile on machine):
- Install Rust => see https://www.rust-lang.org/tools/install
- `cargo install --path .`

## User way:
Download precompiled binary for your platform from Github.

## Usage
```
JSON Lines Viewer - Terminal-UI to view comprehensive application logs in 'Json line format' or Zip files containing such files

Usage: json-lines-viewer [OPTIONS] [FILES]...

Arguments:
  [FILES]...  

Options:
  -f, --field-order <FIELD_ORDER>              fields displayed in-front; separated by comma
  -s, --suppressed-fields <SUPPRESSED_FIELDS>  suppressed fields; separated by comma
  -h, --help                                   Print help
  -V, --version                                Print version
```
