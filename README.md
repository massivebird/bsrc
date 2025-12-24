# bsrc
> short for "bespoke search"

A customizable and fast file querying utility!

🦀 written in Rust

<!-- <p align="center"> -->
<!--   <img width="75%" src="https://imgur.com/b8hfzFN.gif" /> -->
<!-- </p> -->

## How does bsrc work?

By providing bsrc with predefined paths, bsrc knows exactly which directories your queries should (and shouldn't) search. This makes queries inexpensive and blazingly fast.

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

<h3 id="customization">Configuration</h3>

bsrc is configured with `bsrc.toml`.

Here is an example configuration:

```toml
# ./archive/bsrc.toml

# Remove pattern from match names. Optional.
# e.g. '\(.*\)' produces "Metal (USA, EUR)" -> "Metal"
clean = '\(.*\)' 
                 
# Ignore files based on a regex pattern. Optional.
# e.g. '^\.' prevents matching files like `.bios`
ignore = '^\.'

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
