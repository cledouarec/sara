# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.0](https://github.com/cledouarec/sara/compare/sara-core-v0.4.0...sara-core-v0.5.0) - 2026-01-29

### Added

- *(test)* add test_utils module with shared test fixtures (by @cledouarec)
- add ADR (Architecture Decision Record) management (by @cledouarec) - #39

### Changed

- consolidate test helpers and reduce code duplication (by @cledouarec) - #39
- *(graph)* update to use iterator-based returns (by @cledouarec)
- *(error)* add ValidationErrorCode enum for type-safe error codes (by @cledouarec)
- *(model)* add #[must_use] and const fn annotations (by @cledouarec)
- *(cli)* restructure init command to use subcommands (by @cledouarec)
- *(templates)* update templates and remove frontmatter.tera (by @cledouarec)

### Documentation

- update sara-core readme (by @cledouarec) - #37

### Fixed

- *(template)* remove deciders body assertion from ADR test (by @cledouarec)

### Contributors

* @cledouarec

## [0.4.0](https://github.com/cledouarec/sara/compare/sara-core-v0.3.3...sara-core-v0.4.0) - 2026-01-24

### Added

- add depends_on peer dependency support for requirements (by @cledouarec) - #32

### Contributors

* @cledouarec

## [0.3.3](https://github.com/cledouarec/sara/compare/sara-core-v0.3.2...sara-core-v0.3.3) - 2026-01-21

### Other

- update Cargo.toml dependencies

## [0.3.2](https://github.com/cledouarec/sara/compare/sara-core-v0.3.1...sara-core-v0.3.2) - 2026-01-20

### Documentation

- clean changelogs (by @cledouarec) - #23

### Contributors

* @cledouarec

## [0.3.1](https://github.com/cledouarec/sara/compare/sara-core-v0.3.0...sara-core-v0.3.1) - 2026-01-20

### Added

- implement git reference comparison (by @cledouarec) - #20

### Contributors

* @cledouarec

## [0.3.0](https://github.com/cledouarec/sara/compare/sara-core-v0.2.0...sara-core-v0.3.0) - 2026-01-20

### Changed
- move Args and format enums to command modules

## [0.2.0](https://github.com/cledouarec/sara/compare/sara-core-v0.1.3...sara-core-v0.2.0) - 2026-01-20

### Changed
- remove unused line tracking from SourceLocation
- consolidate field names into FieldName enum

## [0.1.2](https://github.com/cledouarec/sara/compare/sara-core-v0.1.1...sara-core-v0.1.2) - 2026-01-16

### Other

- add readme for all crates

## [0.1.1](https://github.com/cledouarec/sara/compare/sara-core-v0.1.0...sara-core-v0.1.1) - 2026-01-16

### Added

- implement SARA requirements knowledge graph CLI