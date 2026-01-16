# sara-core

Core library for [SARA](https://github.com/cledouarec/sara) - **S**olution **A**rchitecture **R**equirement for **A**lignment.

This crate provides the business logic for managing architecture documents and requirements as an interconnected knowledge graph.

## Features

- **Knowledge Graph** - Build and traverse a graph of requirements and architecture documents
- **Markdown Parsing** - Parse Markdown files with YAML frontmatter
- **Multi-Repository Support** - Aggregate documents from multiple Git repositories
- **Validation** - Detect broken references, orphaned items, circular dependencies, and duplicate identifiers
- **Traceability Queries** - Traverse upstream or downstream relationships
- **Reports** - Generate coverage reports and traceability matrices
- **Version Comparison** - Compare knowledge graphs between Git commits or branches
- **Templates** - Generate YAML frontmatter templates for new documents

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
sara-core = "0.1"
```

### Example

```rust,no_run
use std::path::Path;
use sara_core::{
    config::load_config,
    repository::parse_directory,
    graph::GraphBuilder,
    validation::Validator,
    traverse_upstream,
    ItemId, TraversalOptions,
};

fn main() -> sara_core::Result<()> {
    // Load configuration
    let config = load_config(Path::new("sara.toml"))?;

    // Discover and parse documents from a directory
    let items = parse_directory(Path::new("./docs"))?;

    // Build the knowledge graph
    let graph = GraphBuilder::new()
        .add_items(items)
        .build()?;

    // Validate integrity
    let validator = Validator::new(config.validation);
    let report = validator.validate(&graph);

    println!("Errors: {}, Warnings: {}", report.error_count(), report.warning_count());

    // Query traceability
    let options = TraversalOptions::default();
    let item_id = ItemId::new("SYSREQ-001")?;
    if let Some(result) = traverse_upstream(&graph, &item_id, &options) {
        println!("Found {} upstream items", result.items.len());
    }

    Ok(())
}
```

## Document Types

The library supports 9 document types forming a requirements hierarchy:

| Type | Description |
|------|-------------|
| Solution | Customer-facing solution |
| Use Case | Customer/market need |
| Scenario | Abstract system behavior |
| System Requirement | Quantifiable system-level need |
| System Architecture | Platform implementation |
| Hardware Requirement | Hardware-specific need |
| Software Requirement | Software-specific need |
| HW Detailed Design | Hardware implementation |
| SW Detailed Design | Software implementation |

## Traceability Hierarchy

```
Solution
  -> Use Case
      -> Scenario
          -> System Requirement
              -> System Architecture
                  -> Hardware Requirement
                  |     -> HW Detailed Design
                  -> Software Requirement
                        -> SW Detailed Design
```

## License

Licensed under the Apache-2.0 License. See [LICENSE](https://github.com/cledouarec/sara/blob/main/LICENSE) for details.
