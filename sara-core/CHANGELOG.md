# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.7.2](https://github.com/cledouarec/sara/compare/sara-core-v0.7.1...sara-core-v0.7.2) - 2026-03-26

### Styling

- reorder imports (by @cledouarec) - #67

### Contributors

* @cledouarec

## [0.7.1](https://github.com/cledouarec/sara/compare/sara-core-v0.7.0...sara-core-v0.7.1) - 2026-03-26

### Other

- update Cargo.toml dependencies

## [0.7.0](https://github.com/cledouarec/sara/compare/sara-core-v0.6.0...sara-core-v0.7.0) - 2026-03-25

### Changed

- avoid cloning items in KnowledgeGraphBuilder::build (by @cledouarec) - #61
- inline ItemType field-check methods using matches! (by @cledouarec) - #61
- use From<git2::Error> conversion instead of manual map_err in git.rs (by @cledouarec) - #61
- extract helper for repetitive relationship building in yaml parser (by @cledouarec) - #61
- remove dead code from sara-core crate (by @cledouarec) - #61
- move query module into graph as KnowledgeGraph methods (by @cledouarec) - #61
- remove dead code from parser module (by @cledouarec) - #61
- remove dead code from graph module (by @cledouarec) - #61

### Fixed

- add missing assert! in git ref parsing tests (by @cledouarec) - #61

### Contributors

* @cledouarec

## [0.6.0](https://github.com/cledouarec/sara/compare/sara-core-v0.5.4...sara-core-v0.6.0) - 2026-03-24

### Changed

- introduce TemplateRegistry with frontmatter partials (by @cledouarec) - #59
- extract ids_to_relationships helper and clean up service module (by @cledouarec) - #57
- consolidate diff, edit, init modules into service module (by @cledouarec) - #57
- extract adr and builder submodules from model/item (by @cledouarec) - #57
- extract parser/yaml submodule from markdown parser (by @cledouarec) - #57
- replace template module with generator module (by @cledouarec) - #57
- replace upstream/downstream with relationships Vec on Item (by @cledouarec) - #57
- consolidate error types into a single SaraError enum (by @cledouarec) - #57

### Fixed

- resolve clippy warnings and wire up supersedes relationships (by @cledouarec) - #57

### Contributors

* @cledouarec

## [0.5.4](https://github.com/cledouarec/sara/compare/sara-core-v0.5.3...sara-core-v0.5.4) - 2026-03-24

### Other

- update Cargo.toml dependencies

## [0.5.3](https://github.com/cledouarec/sara/compare/sara-core-v0.5.2...sara-core-v0.5.3) - 2026-02-26

### Other

- update Cargo.toml dependencies

## [0.5.2](https://github.com/cledouarec/sara/compare/sara-core-v0.5.1...sara-core-v0.5.2) - 2026-02-18

### Other

- update Cargo.toml dependencies

## [0.5.1](https://github.com/cledouarec/sara/compare/sara-core-v0.5.0...sara-core-v0.5.1) - 2026-02-05

### Other

- update Cargo.toml dependencies

## [0.5.0](https://github.com/cledouarec/sara/compare/sara-core-v0.4.0...sara-core-v0.5.0) - 2026-02-03

### Added

- *(test)* add test_utils module with shared test fixtures (by @cledouarec)
- add ADR (Architecture Decision Record) management (by @cledouarec) - #39

### Changed

- simplify config module by inlining functions (by @cledouarec) - #44
- remove location fields from ValidationError variants (by @cledouarec) - #44
- use KnowledgeGraphBuilder and remove unused functions (by @cledouarec) - #44
- [**breaking**] consolidate parse and validate commands into check (by @cledouarec) - #44
- update validation rules to use ValidationRule trait (by @cledouarec) - #44
- extract find_similar_ids_scored helper (by @cledouarec) - #44
- consolidate test helpers into test_utils module (by @cledouarec) - #44
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