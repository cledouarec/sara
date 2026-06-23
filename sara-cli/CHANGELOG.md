# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.9.1](https://github.com/cledouarec/sara/compare/sara-cli-v0.9.0...sara-cli-v0.9.1) - 2026-06-23

### Other

- update Cargo.lock dependencies

## [0.9.0](https://github.com/cledouarec/sara/compare/sara-cli-v0.8.1...sara-cli-v0.9.0) - 2026-06-12

### Added

- [**breaking**] drive edit, prompts and validation from the active schema (by @cledouarec) - #105
- [**breaking**] a custom schema replaces the built-in model entirely (by @cledouarec) - #105
- add a schema command exporting the active model (by @cledouarec) - #105
- build init subcommands from the active schema (by @cledouarec) - #105
- make relations schema-defined at runtime (by @cledouarec) - #105
- make item types schema-defined at runtime (by @cledouarec) - #105
- drive markdown generation from the active schema (by @cledouarec) - #105
- delegate ItemType/RelationshipRules to active schema (by @cledouarec) - #105

### Changed

- move graph loading into the core service layer (by @cledouarec) - #105
- build the knowledge graph through one helper (by @cledouarec) - #105
- name and share the cli exit codes (by @cledouarec) - #105
- borrow paths as &Path in function signatures (by @cledouarec) - #105

### Fixed

- label query relationships by their schema relation (by @cledouarec) - #105
- [**breaking**] scan only configured repository paths at git refs (by @cledouarec) - #105
- [**breaking**] resolve revision expressions like HEAD~1 as git refs (by @cledouarec) - #105
- [**breaking**] surface skipped files and schema-load failures (by @cledouarec) - #105
- stop panicking when JSON output serialization fails (by @cledouarec) - #105
- accept architecture_decision_record in query --type filter (by @cledouarec) - #105

### Other

- [**breaking**] avoid redundant clones in graph diff and edit merge (by @cledouarec) - #105

### Styling

- align edit and init output on colorize, drop decorative rules (by @cledouarec) - #105

### Testing

- write the schema path TOML-safely in custom-schema configs (by @cledouarec) - #105
- cover the diff command end to end (by @cledouarec) - #105

### Contributors

* @cledouarec

## [0.8.1](https://github.com/cledouarec/sara/compare/sara-cli-v0.8.0...sara-cli-v0.8.1) - 2026-05-31

### Other

- update Cargo.toml dependencies

## [0.8.0](https://github.com/cledouarec/sara/compare/sara-cli-v0.7.6...sara-cli-v0.8.0) - 2026-05-07

### Other

- update Cargo.lock dependencies

## [0.7.6](https://github.com/cledouarec/sara/compare/sara-cli-v0.7.5...sara-cli-v0.7.6) - 2026-04-17

### Other

- release v0.7.6 (by @github-actions[bot]) - #78

### Contributors

* @github-actions[bot]

## [0.7.5](https://github.com/cledouarec/sara/compare/sara-cli-v0.7.4...sara-cli-v0.7.5) - 2026-04-15

### Other

- update Cargo.lock dependencies

## [0.7.4](https://github.com/cledouarec/sara/compare/sara-cli-v0.7.3...sara-cli-v0.7.4) - 2026-04-09

### Fixed

- use config file strict_mode for validation check command (by @cledouarec) - #73
- export full Item in JSON check output instead of incomplete ItemExport (by @cledouarec) - #73

### Contributors

* @cledouarec

## [0.7.3](https://github.com/cledouarec/sara/compare/sara-cli-v0.7.2...sara-cli-v0.7.3) - 2026-04-04

### Build

- remove useless dependencies (by @cledouarec) - #71

### Contributors

* @cledouarec

## [0.7.2](https://github.com/cledouarec/sara/compare/sara-cli-v0.7.1...sara-cli-v0.7.2) - 2026-04-02

### Styling

- reorder imports (by @cledouarec) - #67

### Contributors

* @cledouarec

## [0.7.1](https://github.com/cledouarec/sara/compare/sara-cli-v0.7.0...sara-cli-v0.7.1) - 2026-03-26

### Other

- update Cargo.lock dependencies

## [0.7.0](https://github.com/cledouarec/sara/compare/sara-cli-v0.6.0...sara-cli-v0.7.0) - 2026-03-25

### Changed

- move query module into graph as KnowledgeGraph methods (by @cledouarec) - #61

### Contributors

* @cledouarec

## [0.6.0](https://github.com/cledouarec/sara/compare/sara-cli-v0.5.4...sara-cli-v0.6.0) - 2026-03-24

### Changed

- consolidate diff, edit, init modules into service module (by @cledouarec) - #57
- replace upstream/downstream with relationships Vec on Item (by @cledouarec) - #57
- consolidate error types into a single SaraError enum (by @cledouarec) - #57

### Contributors

* @cledouarec

## [0.5.4](https://github.com/cledouarec/sara/compare/sara-cli-v0.5.3...sara-cli-v0.5.4) - 2026-03-24

### Other

- update Cargo.toml dependencies

## [0.5.3](https://github.com/cledouarec/sara/compare/sara-cli-v0.5.2...sara-cli-v0.5.3) - 2026-02-26

### Other

- update Cargo.toml dependencies

## [0.5.2](https://github.com/cledouarec/sara/compare/sara-cli-v0.5.1...sara-cli-v0.5.2) - 2026-02-18

### Other

- update Cargo.toml dependencies

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