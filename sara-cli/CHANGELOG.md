# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.1](https://github.com/cledouarec/sara/compare/sara-cli-v0.5.0...sara-cli-v0.5.1) - 2026-02-05

### Other

- update Cargo.toml dependencies

## [0.5.0](https://github.com/cledouarec/sara/compare/sara-cli-v0.4.0...sara-cli-v0.5.0) - 2026-02-03

### Added

- add ADR (Architecture Decision Record) management (by @cledouarec) - #39

### Changed

- use KnowledgeGraphBuilder and remove unused functions (by @cledouarec) - #44
- [**breaking**] consolidate parse and validate commands into check (by @cledouarec) - #44
- update validation rules to use ValidationRule trait (by @cledouarec) - #44
- consolidate test helpers and reduce code duplication (by @cledouarec) - #39
- *(cli)* restructure init command to use subcommands (by @cledouarec)
- *(templates)* update templates and remove frontmatter.tera (by @cledouarec)

### Fixed

- *(cli)* change error output to stdout, summaries to stderr (by @cledouarec)

### Contributors

* @cledouarec

## [0.4.0](https://github.com/cledouarec/sara/compare/sara-cli-v0.3.3...sara-cli-v0.4.0) - 2026-01-24

### Added

- add depends_on peer dependency support for requirements (by @cledouarec) - #32

### Documentation

- add gifs in readme (by @cledouarec) - #34

### Contributors

* @cledouarec

## [0.3.3](https://github.com/cledouarec/sara/compare/sara-cli-v0.3.2...sara-cli-v0.3.3) - 2026-01-21

### Other

- update Cargo.toml dependencies

## [0.3.2](https://github.com/cledouarec/sara/compare/sara-cli-v0.3.1...sara-cli-v0.3.2) - 2026-01-20

### Documentation

- clean changelogs (by @cledouarec) - #23

### Contributors

* @cledouarec

## [0.3.1](https://github.com/cledouarec/sara/compare/sara-cli-v0.3.0...sara-cli-v0.3.1) - 2026-01-20

### Added

- implement git reference comparison (by @cledouarec) - #20

### Contributors

* @cledouarec

## [0.3.0](https://github.com/cledouarec/sara/compare/sara-cli-v0.2.0...sara-cli-v0.3.0) - 2026-01-20

### Changed
- move Args and format enums to command modules

## [0.2.0](https://github.com/cledouarec/sara/compare/sara-cli-v0.1.3...sara-cli-v0.2.0) - 2026-01-20

### Changed
- consolidate field names into FieldName enum
- merge prompt_item_id into prompt_identifier

## [0.1.3](https://github.com/cledouarec/sara/compare/sara-cli-v0.1.2...sara-cli-v0.1.3) - 2026-01-16

### Fixed

- colorize text prefix when emojis are disabled
- apply config file output settings

## [0.1.2](https://github.com/cledouarec/sara/compare/sara-cli-v0.1.1...sara-cli-v0.1.2) - 2026-01-16

### Other

- add readme for all crates

## [0.1.1](https://github.com/cledouarec/sara/compare/sara-cli-v0.1.0...sara-cli-v0.1.1) - 2026-01-16

### Added

- implement SARA requirements knowledge graph CLI