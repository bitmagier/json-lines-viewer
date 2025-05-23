JSON Lines Viewer
---
A terminal-UI to browse through JSON-line files.
The main use case is to support the analysis of comprehensive application logs in 'Json line' format.


## Install

### Developer way (compile on machine):
- Install Rust => see https://www.rust-lang.org/tools/install
- `git clone https://github.com/bitmagier/json-lines-viewer.git`
- `cargo install --path .`

### Other way:
Download precompiled binary for your platform from Github.

## Usage

### Run
```
JSON Lines Viewer - Terminal-UI to view comprehensive application logs in 'Json line format' or Zip files containing such files

Usage: json-lines-viewer [OPTIONS] [FILES]...

Arguments:
  [FILES]...  JSON line input files - `.json` or `.zip` files(s) containing `.json` files

Options:
  -f, --field-order <FIELD_ORDER>              fields displayed in-front; separated by comma
  -s, --suppressed-fields <SUPPRESSED_FIELDS>  suppressed fields; separated by comma
  -h, --help                                   Print help
  -V, --version                                Print version
```

### Example
```
json-lines-viewer --field-order @timestamp,level,application_id,message,application_version,land,host_ipv4,host_name,thread_name,correlation_id,logger_name logs-export-XXXX.zip
```


## Program navigation / usage

- Use Cursor Keys, PageUp/PageDown and Left/Right to navigate on a page
- Use `Enter` to go into details of a selected line and `Esc` to go back to a parent screen (also exits program on Main screen)
- Use `Ctrl-S` to save current settings. The current settings are coming from commandline options (or a previously saved config file).   
