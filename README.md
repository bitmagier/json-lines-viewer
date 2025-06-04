JSON Lines Viewer
---
Terminal-UI to view JSON line files (e.g. application logs) or Zip files containing such files

_The main use case is to support the analysis of comprehensive application logs in 'JSON line' format._


## Install

### Rust developer way (compile on machine):
- Install Rust => see https://www.rust-lang.org/tools/install
- `cargo +nightly install json-lines-viewer`

### User way:
Download precompiled binary for your platform from GitHub.

## Usage
```
JSON Lines Viewer – Terminal-UI to view JSON line files (e.g. application logs) or Zip files containing such files

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

Program Navigation:
  * Use cursor keys and page keys to scroll on a screen
  * `Enter` opens a detail screen for the selected line; `Esc` goes back to the parent screen (also exits program on main screen)
  * Use `Ctrl-f` to open a Find dialog; `Esc` leaves the Find dialog; `down/up` jumps to the next/previous finding; a match/miss is indicated by green/red brackets
  * Use `Ctrl-s` to save current settings. Actual settings are always coming from commandline options and the config file if it exists
```

### Example
```
json-lines-viewer --field-order @timestamp,level,application_id,message,application_version,host_ipv4 logs-export-xxxxx.zip
```

