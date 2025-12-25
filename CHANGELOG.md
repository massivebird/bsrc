# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.1] - Unreleased

### Added

+ Added a warning if a directory path does not exist.

### Changed

+ Changed `bsrc.toml` searching behavior, now searches in parent directories if config cannot be found. (#15)

### Fixed

+ Hotfixed `--no-clean` option.
+ Hotfixed `--no-ignore` option.

## [0.2.0] - 2025-12-25

### Changed

+ Changed `dirs.<id>.color` format to a string of hex characters. Example colors:
  + `color = "#FF0000"` (red)
  + `color = "00aabb"` (teal, omitting the leading `#` character)

## [0.1.2] - 2025-12-24

### Added

+ Added `output_fmt` config field for customizable output formatting. (#8)
+ Added `bsrc completions <shell>` subcommand for generating shell completion files.

## [0.1.1] - 2025-12-24

### Added

+ Added `--no-ignore` flag, suppressing the optional `ignore` pattern defined in `bsrc.toml`. (#11)
+ Added `--no-clean` flag, suppressing the optional `clean` pattern defined in `bsrc.toml`. (#12)
