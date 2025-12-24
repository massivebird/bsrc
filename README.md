# bsrc
> short for "bespoke search"

A customizable file search utility!

🦀 written in Rust

<!-- <p align="center"> -->
<!--   <img width="75%" src="https://imgur.com/b8hfzFN.gif" /> -->
<!-- </p> -->

## Elevator pitch

Native OS file searching is slow, unordered, and difficult to customize. bsrc solves all of that.

bsrc searches for file/directory names using a regex pattern, _only_ in paths defined in a TOML config file (see: [Configuration](#config)). This results in fast, inexpensive, and uncluttered searches!

bsrc's suite of command line options also let you customize search and output behavior on the fly.

bsrc is especially great for large archives or libraries of games, music, or books!

## Building

To manually build the project, you must first [install Rust](https://www.rust-lang.org/tools/install).

Once you have Rust installed, run the following commands:

```bash
$ git clone https://github.com/massivebird/bsrc
$ cd bsrc

$ cargo run           # unoptimized build
# OR
$ cargo run --release # optimized build
```

### Adding bsrc to your PATH

If you want to add bsrc to your PATH, I recommend building it in release mode for better optimization.

```bash
$ cd bsrc
$ cargo build --release
$ ln -rs ./target/release/bsrc <dir-in-PATH>/bsrc
$ bsrc
```

## Usage

Basic bsrc syntax is as follows:

```bash
bsrc <query>
```

For more information, run `bsrc --help`.

<h3 id="config">Configuration</h3>

bsrc is configured with `bsrc.toml`.

Here is an example configuration:

```toml
# ./archive/bsrc.toml

# Optional: remove pattern from match names.
# e.g. '\(.*\)' produces "Metal (USA, EUR)" -> "Metal"
clean = '\(.*\)' 
                 
# Optional: ignore files based on a regex pattern.
# e.g. '^\.' prevents matching files like `.bios`
ignore = '^\.'

# Optional: custom output format.
# `%p`: prefix
# `%f`: matching file
output_fmt = '%p :: %f'

# Directory header format: dirs.<unique-id>
[dirs.snes]
prefix = "SNES"
path = "snes" # ./archive/snes/
# Optional fields:
color = [95,0,255] # prefix color
match_dirs = false # if true, matches directories instead of files

[dirs.gba]
prefix = "GBA"
color = [255,175,255]
path = "gba" # ./archive/gba/
match_dirs = false
```
