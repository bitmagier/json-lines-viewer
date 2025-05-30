JSON Lines Viewer
---
Terminal-UI to view JSON line files (e.g. application logs) or Zip files containing such files

_The main use case is to support the analysis of comprehensive application logs in 'Json line' format._


## Install

### Rust developer way (compile on machine):
- Install Rust => see https://www.rust-lang.org/tools/install
- `cargo +nightly install json-lines-viewer`

### User way:
Download precompiled binary for your platform from Github.

## Usage

```
JSON Lines Viewer - Terminal-UI to view '.json' line files (e.g. application logs) or Zip files containing such files

Navigation: Cursor keys, PageUp/Down, Enter/Esc.
Search content: Ctrl-f or '/' and navigate to next/previous finding via cursor down/up. Leave search field with `Esc`.
Save current settings: Ctrl-s (Settings may come from commandline options and a previously saved config file)

Usage: json-lines-viewer [OPTIONS] [FILES]...

Arguments:
  [FILES]...
          JSON line input files - `.json` or `.zip` files(s) containing `.json` files

Options:
  -f, --field-order <FIELD_ORDER>
          fields displayed in-front; separated by comma

  -s, --suppressed-fields <SUPPRESSED_FIELDS>
          suppressed fields; separated by comma

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

### Example
```
json-lines-viewer --field-order @timestamp,level,application_id,message,application_version,land,host_ipv4,host_name,thread_name,correlation_id,logger_name logs-export-xxxxx.zip
```

## Program navigation / usage

- Use Cursor Keys and PageUp/PageDown to navigate on a page
- Use `Enter` to go into details of a selected line and `Esc` to go back to a parent screen (also exits program on Main screen)
- Use `Ctrl-f` opens a find dialog on the bottom to find lines containing a string. `Esc` leaves the find dialog. Use cursor Down/Up here to navigate to the next/previous finding.
- Use `Ctrl-s` to save current settings. The actual settings are always coming from commandline options and a previously saved config file.
