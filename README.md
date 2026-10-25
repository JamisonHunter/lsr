# LSR

A lightweight CLI tool made in Rust for browsing local files with recursive file sizing.

## Motivation

I wanted a very simple CLI tool that I could use to check directory sizes rather than using more complex software just to check a folder size quickly. 

## Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)

### Build & Install For Linux

1. Clone the repository:
   ```bash
   git clone https://github.com/JamisonHunter/lsr
   ```

2. Navigate to your cloned directory and cd into lsr.
    ```bash
    cd lsr
    ```

3. Next build the release version with cargo.
    ```bash
    cargo build --release
    ```

4. Move the executable into your path to make it accessible via the terminal.
    ```bash
    sudo mv target/release/lsr /usr/local/bin/
    ```

5. Lastly, type 'lsr' into the terminal in order to check if it is working. 

## Usage

```
Usage: lsr [OPTION]

Options:
  -a             Show all files including dot files
  -h, --help     Show this help message
  -v, --version  Show version information
```

lsr always scans the directory you are currently in. It accepts no positional
arguments; any extra arguments are rejected. With no options it lists all
non-dot entries sorted largest to smallest. Directory sizes are the recursive
total of the logical file lengths inside them.

### How sizes are measured

- Sizes are logical file lengths (`metadata.len()`), not allocated disk blocks.
- Units are decimal: 1 kB = 1,000 bytes, 1 MB = 1,000 kB, and so on through TB.
- Directory metadata itself is not included in directory sizes.
- Sparse files report their apparent length, which can be much larger than the
  disk space they actually consume.
- Hard links are counted once per directory entry, so the same file content can
  be counted more than once.
- Symbolic links are never followed. A link is listed as a file, and its own
  metadata length is counted rather than the size of its target.
- Dotfiles are included in the grand total even when `-a` is omitted. The `-a`
  option only controls which entries are displayed.

### Errors and exit status

If a path cannot be read, lsr prints a path-specific warning to stderr, keeps
the bytes it was able to count, and marks the affected entry and the grand total
as `(incomplete)`. Entries whose metadata cannot be read are reported and
omitted from the listing. Control characters in file names are escaped so they
cannot break the output layout or inject terminal escape sequences.

| Status | Meaning |
| --- | --- |
| 0 | Scan complete. |
| 1 | Scan incomplete; one or more paths could not be read. |
| 2 | Usage error or fatal error. |

Configuration warnings are reported on stderr but do not change the exit
status.

### Colors

Colors are only emitted when the relevant output stream is a terminal. Set
`NO_COLOR` to disable colors regardless of terminal support. `TERM=dumb` also
disables them.

## Configuration

Colors are read at runtime from `./.lsr.conf` in the directory you run lsr in.
The repository includes a `.lsr.conf` with the default colors, so you can edit it
without recompiling. To use a config from another location, set `LSR_CONFIG`:

```bash
LSR_CONFIG=~/my-lsr.conf lsr
```

If no config file is found, the built-in defaults are used. The file uses simple
`key = value` lines. Blank lines and lines starting with `#` are ignored. A
trailing `# comment` is also ignored when the `#` is preceded by whitespace.

```conf
# Colors accept: #RRGGBB, #RGB, named ANSI colors, 0-255, or "default"

total = #5FB88F                    # "Current directory size" line
directory = #7CC4FF                # directory names
file = default                     # regular file names

progress_fill = bright_cyan        # completed part of the bar
progress_empty = bright_blue       # remaining part of the bar
```

Named colors are `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`,
`white`, and their `bright_` variants. Numeric values use the ANSI 256-color
palette. Invalid settings are reported on stderr and skipped; the previous or
default value is kept.
