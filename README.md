# Lsr

Lsr is a Rust CLI tool that functions similarly to ls with the specific goal of displaying directory sizes through recursion. 

### Motivation

I wanted a very simple CLI tool that I could use to check directory sizes rather than using more complex software just to check a folder size quickly. 

## Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)

### Build and Install

1. Clone the repository:
   ```bash
   git clone https://github.com/JamisonHunter/lsr

2. Navigate to your cloned directory and cd into lsr.
    ```bash
    cd lsr

3. Next build the release version with cargo.
    ```bash
    cargo build --release

4. Move the executable into your path to make it accessible via the terminal.
    ```bash
    sudo mv target/release/lsr /usr/local/bin/

5. Lastly, type 'lsr' into the terminal in order to check if it is working. 
