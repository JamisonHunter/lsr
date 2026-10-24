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

lsr always scans the directory you are currently in. Run it with no arguments to
list everything in that directory, sorted largest to smallest. Directory sizes are the recursive total of everything inside of them, so you can see how much space a folder is really using.

## Configuration

Colors are read at runtime from `./.lsr.conf` in the directory you run lsr in.
The repository includes a `.lsr.conf` with the default colors, so you can edit it
without recompiling. To use a config from another location, set `LSR_CONFIG`:

```bash
LSR_CONFIG=~/my-lsr.conf lsr
```

If no config file is found, the built-in defaults are used. The file uses simple
`key = value` lines; blank lines and lines starting with `#` are ignored.

```conf
# Colors accept: #RRGGBB, #RGB, named ANSI colors, 0-255, or "default"

total = #5FB88F                    # "Current directory storage" line
directory = #7CC4FF                # directory names
file = default                     # regular file names

progress_fill = bright_cyan        # completed part of the bar
progress_empty = bright_blue       # remaining part of the bar
```

Named colors are `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`,
`white`, and their `bright_` variants. Numeric values use the ANSI 256-color
palette. Invalid settings are reported on stderr and skipped.
