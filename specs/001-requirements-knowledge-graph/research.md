# Research: Requirements Knowledge Graph CLI

**Phase**: 0 - Research & Technology Decisions
**Date**: 2026-01-11
**Branch**: `001-requirements-knowledge-graph`

## Technology Decisions

### 1. Graph Library: Petgraph

**Decision**: Use `petgraph` for in-memory graph representation.

**Rationale**:
- De facto standard graph library for Rust with mature, stable API
- Supports directed graphs required for "requires"/"realizes" relationships
- Built-in cycle detection via `is_cyclic_directed()` (FR-013)
- Efficient traversal algorithms (DFS, BFS) for upstream/downstream queries (FR-016, FR-017)
- Serialization support via serde feature for potential caching

**Alternatives Considered**:
| Alternative | Reason Rejected |
|-------------|-----------------|
| Custom graph implementation | Unnecessary complexity, would require implementing cycle detection and traversal algorithms |
| `daggy` | More specialized for DAGs, but petgraph is more flexible and widely used |
| `graphlib` | Less maintained than petgraph |

**Configuration**:
```rust
// Use DiGraph for directed relationships
use petgraph::graph::DiGraph;
type KnowledgeGraph = DiGraph<Item, Relationship>;
```

---

### 2. CLI Framework: Clap v4

**Decision**: Use `clap` with derive macros for CLI argument parsing.

**Rationale**:
- Industry standard for Rust CLIs with excellent documentation
- Derive macros reduce boilerplate and ensure consistency
- Built-in help generation, shell completions, and error messages
- Subcommand support matches our command structure (parse, validate, query, report)
- Integrates well with colored output

**Alternatives Considered**:
| Alternative | Reason Rejected |
|-------------|-----------------|
| `structopt` | Deprecated in favor of clap derive |
| `argh` | Less feature-rich, smaller ecosystem |
| Manual parsing | Significant boilerplate, error-prone |

**Pattern**:
```rust
#[derive(Parser)]
#[command(name = "sara", about = "Requirements Knowledge Graph CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true)]
    verbose: bool,
}
```

---

### 3. Markdown Parsing: pulldown-cmark

**Decision**: Use `pulldown-cmark` for Markdown parsing.

**Rationale**:
- CommonMark compliant, fast pull-based parser
- Only need to extract YAML frontmatter, not render full Markdown
- Lightweight with minimal dependencies
- Well-maintained by the Rust community

**Alternatives Considered**:
| Alternative | Reason Rejected |
|-------------|-----------------|
| `comrak` | More features than needed, heavier dependency |
| `markdown-rs` | Less mature |
| Manual regex parsing | Fragile, error-prone for edge cases |

**Usage Pattern**:
- Parse frontmatter between `---` delimiters
- Use pulldown-cmark only if we need to extract inline references from body text
- Primary metadata comes from YAML frontmatter via serde_yaml

---

### 4. YAML Frontmatter: serde + serde_yaml

**Decision**: Use `serde` with `serde_yaml` for frontmatter deserialization.

**Rationale**:
- Standard Rust serialization framework with excellent derive support
- `serde_yaml` handles YAML parsing with good error messages
- Type-safe deserialization into domain structs
- Builder pattern integration via `#[serde(default)]` and custom deserializers

**Configuration**:
```rust
#[derive(Debug, Deserialize)]
struct Frontmatter {
    id: String,
    #[serde(rename = "type")]
    item_type: ItemType,
    name: String,
    #[serde(default)]
    requires: Vec<String>,
    #[serde(default)]
    realizes: Vec<String>,
}
```

---

### 5. Git Operations: git2

**Decision**: Use `git2` (libgit2 bindings) for Git repository operations.

**Rationale**:
- Read files from specific commits/branches (FR-029)
- Compare graphs between Git references (FR-030)
- No need to shell out to `git` command
- Cross-platform support (FR-031)

**Alternatives Considered**:
| Alternative | Reason Rejected |
|-------------|-----------------|
| `gix` (gitoxide) | More complex API, still maturing |
| Shell commands | Platform-dependent, harder to parse output |

**Usage Pattern**:
```rust
// Read file at specific commit
let repo = Repository::open(path)?;
let commit = repo.revparse_single(ref_spec)?.peel_to_commit()?;
let tree = commit.tree()?;
let entry = tree.get_path(Path::new("docs/REQ-001.md"))?;
```

---

### 6. Error Handling: thiserror

**Decision**: Use `thiserror` for error type definitions.

**Rationale**:
- Minimal boilerplate for custom error types
- Automatic `Display` and `Error` trait implementations
- Source error chaining for context
- Works well with `anyhow` if needed in CLI layer

**Pattern**:
```rust
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Invalid frontmatter in {file} at line {line}: {reason}")]
    InvalidFrontmatter { file: PathBuf, line: usize, reason: String },

    #[error("Missing required field '{field}' in {file}")]
    MissingField { file: PathBuf, field: String },
}
```

---

### 7. Terminal Output: colored + console

**Decision**: Use `colored` for ANSI colors and `console` for emoji support.

**Rationale**:
- FR-027 requires colored output with emojis
- `colored` provides simple color macros
- `console` provides terminal feature detection and emoji rendering
- Both handle Windows terminal compatibility

**Pattern**:
```rust
use colored::*;
use console::Emoji;

println!("{} Validation passed", Emoji("✅", "[OK]"));
println!("{}: {}", "Error".red().bold(), message);
```

---

### 8. Logging: tracing + tracing-subscriber

**Decision**: Use `tracing` for structured logging and diagnostics.

**Rationale**:
- FR-026 requires configurable logging levels
- `tracing` is the modern replacement for `log`, with structured spans and events
- Better async support and performance instrumentation
- `tracing-subscriber` provides flexible output formatting
- Compatible with `log` crate via `tracing-log` if needed for dependencies

**Alternatives Considered**:
| Alternative | Reason Rejected |
|-------------|-----------------|
| `log` + `env_logger` | Less structured, no span support, older approach |
| `slog` | More complex API, less ecosystem adoption |

**Configuration Levels**:
| CLI Flag | Log Level | Description |
|----------|-----------|-------------|
| (none) | WARN | Errors and warnings only |
| `-v` | INFO | Progress and key events |
| `-vv` | DEBUG | Detailed operations |
| `-vvv` | TRACE | Full diagnostic output |
| `--quiet` | ERROR | Errors only |

**Usage Pattern**:
```rust
use tracing::{info, debug, warn, error, instrument, span, Level};

#[instrument(skip(graph))]
pub fn validate(graph: &KnowledgeGraph) -> ValidationReport {
    info!("Starting validation of {} items", graph.item_count());
    // ...
}
```

---

### 9. Configuration: toml

**Decision**: Use TOML format for configuration files.

**Rationale**:
- FR-024 requires configuration file support
- TOML is readable, well-suited for configuration
- Good serde support via `toml` crate
- Commonly used in Rust ecosystem (Cargo.toml)

**Configuration File Structure**:
```toml
# sara.toml
[repositories]
paths = [
    "./docs",
    "../hardware-repo/specs",
    "../software-repo/requirements"
]

[validation]
strict_orphans = false  # permissive mode by default

[output]
colors = true
emojis = true
```

---

### 10. Builder Pattern Implementation

**Decision**: Use typed builders for complex object construction.

**Rationale**:
- User specified Builder pattern preference
- Provides clear, fluent API for graph construction
- Enables validation at construction time
- Supports optional fields with sensible defaults

**Pattern Examples**:

```rust
// ItemBuilder for constructing items
let item = ItemBuilder::new("REQ-001", ItemType::SystemRequirement)
    .name("Performance Requirement")
    .description("System must respond within 100ms")
    .requires(vec!["SCEN-001"])
    .build()?;

// GraphBuilder for constructing the knowledge graph
let graph = GraphBuilder::new()
    .add_repository("./docs")?
    .add_repository("../other-repo/specs")?
    .with_strict_orphan_check(true)
    .build()?;

// ValidationReportBuilder for constructing reports
let report = ValidationReportBuilder::new()
    .add_error(ValidationError::BrokenRef { ... })
    .add_warning(ValidationWarning::Orphan { ... })
    .build();
```

---

## Performance Considerations

### Target: 500 documents in <1 second (SC-001)

**Strategies**:

1. **Parallel File Parsing**: Use `rayon` for parallel file discovery and parsing
   ```rust
   documents.par_iter().map(|path| parse_document(path)).collect()
   ```

2. **Lazy Graph Construction**: Build graph edges after all nodes are loaded to avoid repeated lookups

3. **Indexed Lookups**: Use `HashMap<ItemId, NodeIndex>` for O(1) item lookups instead of graph traversal

4. **Early Termination**: For cycle detection, use Petgraph's built-in algorithms which are optimized

5. **Streaming Validation**: Report errors as found rather than collecting all first

**Benchmarking**:
- Add benchmark tests using `criterion` crate
- Target: < 2ms per document average

---

## Cross-Platform Considerations (FR-031)

### Windows Compatibility

1. **Path Handling**: Use `PathBuf` and `Path::join()` consistently
2. **Git Operations**: `git2` handles Windows paths correctly
3. **Terminal Colors**: `colored` and `console` detect Windows terminal capabilities
4. **Line Endings**: Handle both LF and CRLF in Markdown files

### macOS/Linux Compatibility

1. **File Permissions**: Respect Unix file permissions
2. **Symlinks**: Follow symlinks in repository scanning
3. **Unicode**: Full Unicode support in identifiers and file paths

---

## Open Questions Resolved

All technical decisions have been made. No NEEDS CLARIFICATION items remain.

| Question | Resolution |
|----------|------------|
| Which graph library? | Petgraph - mature, full-featured |
| How to handle cross-platform? | git2 + colored/console with platform detection |
| Configuration format? | TOML (Rust ecosystem standard) |
| Error handling strategy? | thiserror for types, anyhow for CLI |
| Performance approach? | Rayon parallelism + indexed lookups |

---

## Interactive Mode Research (Added 2026-01-14)

This section documents research for FR-040 through FR-052 (Interactive Mode).

### 11. Interactive Prompt Library: inquire

**Decision**: Use `inquire` v0.9+ for interactive terminal prompts.

**Rationale**:
- Cross-platform support (Windows, macOS, Linux) via crossterm, matching our target platforms (FR-036)
- Provides all needed prompt types: `Select`, `MultiSelect`, `Text`, `Confirm`
- Active maintenance with 8M+ downloads indicates stability
- Built-in validators (`MinLengthValidator`, `ValueRequiredValidator`) align with FR-047
- Customizable render config for consistent styling with existing CLI output
- Default crossterm backend matches no additional dependencies

**Alternatives Considered**:
| Alternative | Reason Rejected |
|-------------|-----------------|
| `dialoguer` | Uses `console` crate; less customizable validation |
| `cliclack` | Newer, less proven; different visual style |
| `promptly` | Too simple; lacks MultiSelect and validators |

**References**:
- [inquire on crates.io](https://crates.io/crates/inquire)
- [inquire documentation](https://docs.rs/inquire)

**Configuration**:
```toml
# sara-cli/Cargo.toml
[dependencies]
inquire = "0.9"
```

---

### 12. TTY Detection Strategy

**Decision**: Use `std::io::stdin().is_terminal()` from Rust standard library.

**Rationale**:
- Available in Rust 1.70+ (we require 1.75+), no external dependency needed
- Check both stdin and stdout for complete detection
- Exit with clear error message before prompts start, per FR-051

**Alternatives Considered**:
| Alternative | Reason Rejected |
|-------------|-----------------|
| `atty` crate | External dependency; deprecated in favor of std |
| `is-terminal` crate | Unnecessary; std now provides this |
| Let inquire fail | Poor UX; user sees cryptic error instead of guidance |

**Implementation Pattern**:
```rust
use std::io::{stdin, stdout, IsTerminal};

fn require_tty() -> Result<(), InitError> {
    if !stdin().is_terminal() || !stdout().is_terminal() {
        return Err(InitError::NonInteractiveTerminal);
    }
    Ok(())
}
```

**Error Message**: "Interactive mode requires a terminal. Use --type <TYPE> to specify the item type."

---

### 13. Interactive Mode Entry Condition

**Decision**: Enter interactive mode when `--type` is not provided.

**Rationale**:
- Matches FR-040: "when the init command is invoked with only a file name or no arguments"
- Simple mental model: `--type` present = non-interactive, `--type` absent = interactive
- Allows partial interactive mode: user can provide some args and be prompted for rest (FR-050)

**CLI Behavior Matrix**:
| Command | Mode |
|---------|------|
| `sara init doc.md --type solution` | Non-interactive |
| `sara init doc.md` | Interactive |
| `sara init` | Interactive (prompts for file path too) |
| `sara init --name "Foo"` | Interactive (skips name prompt) |

---

### 14. Knowledge Graph Access for Traceability

**Decision**: Parse repositories at interactive mode start.

**Rationale**:
- Ensures graph availability for ID suggestions (FR-044) and traceability lists (FR-045)
- Validates parent items exist before allowing creation (FR-052)
- Performance impact acceptable: parsing is fast (<1s for 500 docs per spec)

**Implementation Notes**:
- Reuse existing `parse_repositories()` function from `sara_core`
- Graph is read-only during interactive session

---

### 15. Prompt Flow and Field Ordering

**Decision**: Prompt in logical dependency order.

**Flow**:
1. **Item Type** (Select) - Required first to determine subsequent prompts
2. **Name** (Text, required) - Human-readable identifier
3. **Identifier** (Text with default) - Suggest next available ID based on type
4. **Description** (Text, optional) - Brief summary
5. **Traceability Links** (MultiSelect) - Populated based on item type:
   - UseCase/Scenario: `refines` (show Solutions/UseCases)
   - Requirements: `derives_from` (show Scenarios/SystemArchitectures)
   - Architectures/Designs: `satisfies` (show Requirements)
6. **Type-specific fields**:
   - Requirements: `specification` (Text, required)
   - SystemArchitecture: `platform` (Text, optional)
7. **Confirmation** (Confirm) - Show summary, allow cancel

---

### 16. ID Generation Strategy

**Decision**: Increment from highest existing ID (e.g., SOL-001, SOL-002).

**Rationale**:
- Matches existing `generate_id()` function in `sara_core`
- Follows established patterns: SOL-001, UC-001, SYSREQ-001
- Requires parsing graph to find max ID per type
- Allow user to override suggested default

**Enhancement for interactive mode**:
- Pass parsed graph to `generate_id()` to find next sequence number
- Display as default value in Text prompt: "Identifier [SOL-003]: "

---

### 17. Parent Item Enforcement

**Decision**: Block creation until required parents exist (per FR-052).

**Rationale**:
- Clarification from spec session resolved this: strict enforcement
- Prevents creation of orphaned items in interactive mode
- Error message lists required parent types

**Error Message**: "Cannot create Scenario: no Use Cases exist. Create a Use Case first."

**Parent Requirements by Type**:
| Item Type | Required Parent Type |
|-----------|---------------------|
| Solution | None (root) |
| UseCase | Solution |
| Scenario | UseCase |
| SystemRequirement | Scenario |
| SystemArchitecture | SystemRequirement |
| HardwareRequirement | SystemArchitecture |
| SoftwareRequirement | SystemArchitecture |
| HardwareDetailedDesign | HardwareRequirement |
| SoftwareDetailedDesign | SoftwareRequirement |

---

### 18. Ctrl+C Handling

**Decision**: Exit cleanly without creating any files (FR-049).

**Implementation**:
- `inquire` returns `InquireError::OperationInterrupted` on Ctrl+C
- Catch at top level of interactive flow
- Print message: "Cancelled. No file was created."
- Exit with code 130 (standard Unix signal exit code: 128 + SIGINT)

---

### 19. Module Organization

**Decision**: New `interactive.rs` module under `sara-cli/src/commands/`.

**Rationale**:
- Keeps `init.rs` focused on non-interactive logic
- Single responsibility: all prompt-related code in one place
- Avoids circular dependencies

**Proposed File Structure**:
```
sara-cli/src/commands/
├── mod.rs           # Add `mod interactive;`
├── init.rs          # Calls interactive module when --type missing
├── interactive.rs   # NEW: All prompt logic
```

---

### 20. Validation and Re-prompting

**Decision**: Inline validation with immediate re-prompt (FR-047).

**Implementation**:
- Use `inquire` built-in validators for simple rules (non-empty, length)
- Custom validator for ID format (alphanumeric, hyphens, underscores)
- No "retry" count limit - keep prompting until valid or cancelled

**Validator Examples**:
```rust
// Name validator
Text::new("Item name:")
    .with_validator(MinLengthValidator::new(1).with_message("Name is required"))
    .prompt()

// ID validator
Text::new("Identifier:")
    .with_default(&suggested_id)
    .with_validator(|input: &str| {
        if input.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            Ok(Validation::Valid)
        } else {
            Ok(Validation::Invalid("ID must contain only letters, numbers, hyphens, and underscores".into()))
        }
    })
    .prompt()
```

---

### Interactive Mode Open Questions Resolved

All clarifications from spec.md have been incorporated:

| Question | Resolution |
|----------|------------|
| Non-TTY handling | Exit with error, instruct user to use CLI flags (Decision 12) |
| Empty traceability options | Block creation until parents exist (Decision 17) |
| Terminal width | No special handling needed; inquire adapts automatically |

---

## Edit Command Research (Added 2026-01-14)

This section documents research for FR-054 through FR-066 (Edit Command).

### 21. Edit Command Architecture

**Decision**: Reuse Interactive Mode infrastructure for Edit Command.

**Rationale**:
- Edit command shares 80% of interactive mode code (prompts, validation, traceability selection)
- Same prompt flow with pre-populated defaults instead of empty fields
- Consistent user experience between `init` and `edit` commands
- Reduces code duplication per Constitution Principle 1 (Simplicity First)

**Key Differences from Init**:
| Aspect | Init Command | Edit Command |
|--------|--------------|--------------|
| Input | File path (optional) | Item ID (required) |
| Mode trigger | Missing `--type` | Missing modification flags |
| Prompt defaults | Empty/generated | Current values |
| Type field | Prompted/required | Read-only (immutable) |
| ID field | Prompted/generated | Read-only (immutable) |
| File creation | Creates new file | Updates existing file |

---

### 22. Item Lookup Strategy

**Decision**: Use existing graph lookup with fuzzy suggestion on failure.

**Rationale**:
- FR-061 requires suggestions for similar identifiers on "not found"
- Reuse existing `KnowledgeGraph::get()` for O(1) lookup
- Use Levenshtein distance for similarity suggestions (already available via strsim crate)

**Implementation Pattern**:
```rust
fn lookup_item_or_suggest(graph: &KnowledgeGraph, id: &str) -> Result<&Item, EditError> {
    if let Some(item) = graph.get(&ItemId::new(id)?) {
        return Ok(item);
    }

    // Find similar IDs for suggestion
    let suggestions: Vec<_> = graph.all_items()
        .filter(|item| strsim::levenshtein(item.id.as_str(), id) <= 3)
        .take(3)
        .map(|item| item.id.as_str())
        .collect();

    Err(EditError::ItemNotFound { id: id.to_string(), suggestions })
}
```

**Error Message Format**:
```
❌ Item not found: SREQ-099
Did you mean: SREQ-001, SREQ-009, SREQ-010?
```

---

### 23. Interactive Edit Mode Entry Condition

**Decision**: Enter interactive mode when no modification flags are provided.

**Rationale**:
- Consistent with init command pattern (FR-055)
- Simple mental model: flags present = apply changes, flags absent = interactive
- Allows non-interactive partial updates via flags

**CLI Behavior Matrix**:
| Command | Mode |
|---------|------|
| `sara edit SREQ-001` | Interactive |
| `sara edit SREQ-001 --name "New Name"` | Non-interactive |
| `sara edit SREQ-001 --derives-from SCEN-002` | Non-interactive |
| `sara edit SREQ-001 --name "X" --description "Y"` | Non-interactive |

---

### 24. Pre-populated Prompt Defaults

**Decision**: Use `inquire` `.with_default()` for all editable fields (FR-056).

**Rationale**:
- User can press Enter to keep current value
- User can type new value to replace
- Consistent with standard CLI editing patterns

**Implementation Pattern**:
```rust
// Name prompt with current value as default
let name = Text::new("Item name:")
    .with_default(&current_item.name)
    .with_validator(MinLengthValidator::new(1))
    .prompt()?;

// MultiSelect with current selections pre-selected
let current_refines: Vec<_> = current_item.upstream.refines.iter()
    .map(|id| id.as_str().to_string())
    .collect();

let refines = MultiSelect::new("Select items to refine:", options)
    .with_default(&current_refines)
    .prompt()?;
```

---

### 25. Immutable Fields Display

**Decision**: Display type and ID as read-only information, not prompts (FR-059, FR-060).

**Rationale**:
- Type and ID are identity fields that must not change
- Changing ID would break all existing references to the item
- Changing type would violate relationship rules

**Display Pattern**:
```
📋 Editing SREQ-001 (System Requirement)

? Item name [Performance Requirement]:
```

---

### 26. Change Summary and Confirmation

**Decision**: Show diff-style summary before applying changes (FR-063).

**Rationale**:
- User must confirm before changes are written
- Clear visualization of what will change
- Consistent with init command confirmation pattern

**Display Pattern**:
```
📋 Changes to apply:

  Name: Performance Requirement → High Performance Requirement
  Description: (unchanged)
  Derives from: SCEN-001 → SCEN-001, SCEN-002

? Apply changes? (Y/n)
```

---

### 27. File Modification Strategy

**Decision**: Preserve body content, update only YAML frontmatter (FR-064).

**Rationale**:
- User's prose content must not be altered
- Only metadata (frontmatter) is managed by the CLI
- Matches init command behavior

**Implementation Pattern**:
```rust
fn update_frontmatter(file_path: &Path, new_frontmatter: &str) -> Result<(), IoError> {
    let content = fs::read_to_string(file_path)?;

    // Extract body (everything after closing ---)
    let body = extract_body(&content);

    // Combine new frontmatter with preserved body
    let updated = format!("---\n{}---\n{}", new_frontmatter, body);

    fs::write(file_path, updated)?;
    Ok(())
}
```

---

### 28. Traceability Validation

**Decision**: Validate new links reference existing items before applying (FR-065).

**Rationale**:
- Prevents creating broken references through edit
- Uses same validation as init command
- Consistent error messages

**Validation Flow**:
1. User selects traceability links in MultiSelect
2. Before applying, verify each selected ID exists in graph
3. If any invalid, show error and re-prompt (interactive) or fail (non-interactive)

---

### 29. Module Organization for Edit

**Decision**: Add `edit.rs` module alongside `interactive.rs` in sara-cli/src/commands/.

**Rationale**:
- Single responsibility: edit command logic in one place
- Reuse interactive module for prompt functions
- Minimal code duplication

**Proposed File Structure**:
```
sara-cli/src/commands/
├── mod.rs           # Add `mod edit;` and Edit command enum
├── init.rs          # Init command (existing)
├── interactive.rs   # Shared prompt functions (existing)
├── edit.rs          # NEW: Edit command implementation
```

**Code Reuse**:
- `interactive::prompt_name()` - reusable with default parameter
- `interactive::prompt_description()` - reusable with default parameter
- `interactive::prompt_traceability()` - reusable with pre-selection
- `interactive::prompt_specification()` - reusable with default parameter
- `interactive::require_tty()` - reusable as-is

---

### 30. Non-Interactive Edit Mode

**Decision**: Apply changes directly when modification flags are provided (FR-057, FR-058).

**Rationale**:
- Enables scripted/automated metadata updates
- Only specified fields are changed, others preserved
- No confirmation prompt needed (explicit flags = explicit intent)

**CLI Examples**:
```bash
# Update only name
sara edit SREQ-001 --name "New Performance Requirement"

# Update traceability (replaces existing)
sara edit SREQ-001 --derives-from SCEN-002 SCEN-003

# Update multiple fields
sara edit SREQ-001 --name "New Name" --specification "The system SHALL..."
```

---

### Edit Command Open Questions Resolved

| Question | Resolution |
|----------|------------|
| How to handle immutable fields? | Display as read-only info, not prompts (Decision 25) |
| What triggers interactive mode? | Absence of modification flags (Decision 23) |
| How to show changes? | Diff-style summary before confirmation (Decision 26) |
| How to find similar IDs? | Levenshtein distance with max 3 suggestions (Decision 22) |
