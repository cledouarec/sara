# Feature Specification: ADR Management with Design Linking

**Feature Branch**: `002-adr-management`
**Created**: 2026-01-26
**Status**: Draft
**Input**: User description: "My application should manage Architecture decision records (ADR) linked to System architecture, Software or Hardware detailed design."

## Clarifications

### Session 2026-01-26

- Q: How are ADRs uniquely identified? → A: Sequential number (ADR-001, ADR-002, etc.)
- Q: Where do design artifacts come from? → A: Managed within this system (artifacts created/stored here)
- Q: How are concurrent edit conflicts handled? → A: Managed by Git; validation fails if ADR number already exists
- Q: Are ADR status transitions constrained? → A: Any transition allowed (flexibility for corrections)
- Q: How are design artifacts identified? → A: Type-prefixed sequential number (SYSARCH-001, SWDD-001, HWDD-001)
- Q: How does ADR integrate with existing data model? → A: New ItemType extending existing KnowledgeGraph; uses reserved `justified_by` field; new Justifies/Supersedes relationship types
- Q: Should ADR store date and ticket in frontmatter? → A: No, Git tracks dates; ticket references go in markdown body (Links section)
- Q: How should the CLI init command be structured? → A: Refactored to use subcommands per item type to reduce complexity

## CLI Refactoring: Init Subcommands

### Problem Statement

The current `sara init` command exposes all possible options regardless of item type, leading to:
- User confusion about which options apply to which item types
- Risk of invalid option combinations
- Complex help output with many type-specific headings

### Solution: Type-Specific Subcommands

Replace the flat `sara init --type <type>` pattern with type-specific subcommands:

```
sara init              # Interactive mode (prompts for item type and fields)
sara init <item_type>  # Subcommand with type-specific options only
```

### Subcommand Structure

| Subcommand | Alias | Item Type | Specific Options |
|------------|-------|-----------|------------------|
| `sara init adr` | - | Architecture Decision Record | `--status`, `--deciders`, `--justifies`, `--supersedes` |
| `sara init system-requirement` | `sysreq` | System Requirement | `--specification`, `--derives-from`, `--depends-on` |
| `sara init system-architecture` | `sysarch` | System Architecture | `--platform`, `--satisfies` |
| `sara init software-requirement` | `swreq` | Software Requirement | `--specification`, `--derives-from`, `--depends-on` |
| `sara init hardware-requirement` | `hwreq` | Hardware Requirement | `--specification`, `--derives-from`, `--depends-on` |
| `sara init software-detailed-design` | `swdd` | Software Detailed Design | `--satisfies` |
| `sara init hardware-detailed-design` | `hwdd` | Hardware Detailed Design | `--satisfies` |
| `sara init solution` | `sol` | Solution | (none) |
| `sara init use-case` | `uc` | Use Case | `--refines` |
| `sara init scenario` | `scen` | Scenario | `--refines` |

### Common Options (All Subcommands)

All subcommands share these base options:
- `<FILE>` - Target markdown file (required)
- `--id` - Item identifier (auto-generated if not provided)
- `--name` - Item name (extracted from title if not provided)
- `--description`, `-d` - Item description
- `--force` - Overwrite existing frontmatter

### Usage Examples

```bash
# Interactive mode - prompts for everything
sara init

# Create an ADR with specific options
sara init adr path/to/decision.md --status proposed --deciders "Alice, Bob"

# Create a system requirement (full name)
sara init system-requirement path/to/req.md --specification "The system SHALL..."

# Create a system requirement (using alias)
sara init sysreq path/to/req.md --specification "The system SHALL..."

# Create a system architecture with platform
sara init system-architecture path/to/arch.md --platform "Linux ARM64" --satisfies SYSREQ-001

# Create a software requirement derived from system requirement (using alias)
sara init swreq path/to/swreq.md --derives-from SYSREQ-001 --specification "The software SHALL..."

# Create a use case (using alias)
sara init uc path/to/usecase.md --refines SOL-001
```

### Benefits

1. **Clearer Help Output**: Each subcommand shows only relevant options
2. **Validation at Parse Time**: Invalid options rejected immediately by clap
3. **Better Discoverability**: Users can run `sara init --help` to see available item types
4. **Reduced Complexity**: No need for runtime validation of option/type compatibility

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Create and Link ADR to Design Artifact (Priority: P1)

As an architect or engineer, I want to create an Architecture Decision Record and link it to one or more design artifacts (system architecture, software design, or hardware design) so that the rationale behind design choices is documented and traceable.

**Why this priority**: This is the core functionality - without the ability to create ADRs and establish links to design artifacts, the feature has no value.

**Independent Test**: Can be fully tested by creating a new ADR document, selecting one or more design artifacts to link, and verifying the link is persisted and visible.

**Acceptance Scenarios**:

1. **Given** a user is working on a project with existing design artifacts, **When** they create a new ADR and select design artifacts to link, **Then** the ADR is created with status "Proposed" and the links to selected artifacts are established.
2. **Given** a user is creating an ADR, **When** they provide the required fields (title, context, decision, consequences), **Then** the ADR is saved with all provided information.
3. **Given** an ADR exists, **When** a user views the ADR, **Then** they see all linked design artifacts with their types (System/Software/Hardware).

---

### User Story 2 - Navigate Between ADRs and Design Artifacts (Priority: P2)

As a team member, I want to navigate from a design artifact to its related ADRs and vice versa so that I can understand the decisions that shaped a particular design.

**Why this priority**: Traceability is a key value proposition - users need bidirectional navigation to understand relationships between decisions and designs.

**Independent Test**: Can be fully tested by viewing a design artifact and clicking through to related ADRs, then from an ADR navigating back to linked artifacts.

**Acceptance Scenarios**:

1. **Given** a design artifact has linked ADRs, **When** a user views the artifact, **Then** they see a list of all ADRs that reference this artifact.
2. **Given** an ADR links to multiple design artifacts, **When** a user views the ADR, **Then** they can navigate directly to any of the linked artifacts.
3. **Given** a design artifact has no linked ADRs, **When** a user views the artifact, **Then** they see an indication that no ADRs reference this artifact.

---

### User Story 3 - Update ADR Status and Lifecycle (Priority: P2)

As an architect, I want to update the status of an ADR (Proposed, Accepted, Deprecated, Superseded) so that the current validity of each decision is clear to all team members.

**Why this priority**: ADRs evolve over time - tracking their lifecycle status is essential for understanding which decisions are currently in effect.

**Independent Test**: Can be fully tested by changing an ADR's status and verifying the change is reflected in all views.

**Acceptance Scenarios**:

1. **Given** an ADR with status "Proposed", **When** an authorized user changes the status to "Accepted", **Then** the status is updated and the change is recorded.
2. **Given** an ADR is being superseded, **When** a user marks it as "Superseded", **Then** they can link it to the superseding ADR.
3. **Given** an ADR has been deprecated, **When** team members view it, **Then** they see a clear visual indication of its deprecated status.

---

### User Story 4 - Search and Filter ADRs (Priority: P3)

As a team member, I want to search and filter ADRs by status, linked artifact type, or keywords so that I can quickly find relevant architectural decisions.

**Why this priority**: As the number of ADRs grows, finding specific decisions becomes challenging without search and filter capabilities.

**Independent Test**: Can be fully tested by creating multiple ADRs with different attributes and verifying search/filter returns correct results.

**Acceptance Scenarios**:

1. **Given** multiple ADRs exist in the system, **When** a user filters by status "Accepted", **Then** only ADRs with that status are displayed.
2. **Given** ADRs are linked to different artifact types, **When** a user filters by "Hardware Design", **Then** only ADRs linked to hardware design artifacts are shown.
3. **Given** a user searches for a keyword, **When** the keyword exists in ADR titles or content, **Then** matching ADRs are returned in the results.

---

### Edge Cases

- What happens when a linked design artifact is deleted? The ADR retains a reference indicating the artifact was removed, preserving historical context.
- How does the system handle an ADR that supersedes multiple other ADRs? The system supports linking to multiple superseded ADRs.
- What happens when a user tries to create an ADR without required fields? The system validates input and displays clear error messages for missing required fields.
- How are ADRs handled when they link to artifacts across different design types? ADRs can link to any combination of system, software, and hardware design artifacts simultaneously.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST allow users to create ADRs with required fields: title, context, decision, and consequences.
- **FR-002**: System MUST support ADR status values: Proposed, Accepted, Deprecated, and Superseded.
- **FR-003**: System MUST allow linking an ADR to one or more design artifacts of types: System Architecture, Software Design, and Hardware Design.
- **FR-004**: System MUST display all linked design artifacts when viewing an ADR.
- **FR-005**: System MUST display all related ADRs when viewing a design artifact.
- **FR-006**: System MUST allow users to update the status of an existing ADR to any valid status (no transition restrictions).
- **FR-007**: System MUST support linking a superseded ADR to its successor ADR.
- **FR-008**: System MUST preserve ADR links as references when linked artifacts are deleted.
- **FR-009**: System MUST provide search capability across ADR titles and content.
- **FR-010**: System MUST provide filtering of ADRs by status and linked artifact type.
- **FR-011**: System MUST track creation and modification dates via Git history (no explicit date field in frontmatter).
- **FR-012**: System MUST validate that required ADR fields are provided before saving.
- **FR-013**: System MUST validate that ADR numbers are unique and fail if a duplicate number is used.

### Key Entities

- **ADR (Architecture Decision Record)**: A document capturing an architectural decision, uniquely identified by a sequential number (ADR-001, ADR-002, etc.). Includes title, context (why the decision is needed), decision (what was decided), consequences (impact of the decision), status, deciders, and links to design artifacts. Creation and modification dates are tracked via Git history.
- **Design Artifact**: A document representing system architecture (SYSARCH-nnn), software detailed design (SWDD-nnn), or hardware detailed design (HWDD-nnn). Each artifact is uniquely identified by a type-prefixed sequential number, has a name, and can be referenced by multiple ADRs.
- **ADR Link**: A relationship connecting an ADR to a design artifact, capturing which designs are impacted by or related to a specific architectural decision.
- **Supersession Relationship**: A special link between two ADRs indicating that one decision replaces another.

### Integration with Existing System

Based on codebase analysis, ADR management integrates with the existing SARA knowledge graph architecture:

**ItemType Extension**:
- ADR becomes a new `ItemType::ArchitectureDecisionRecord` with prefix "ADR"
- Does not fit the standard hierarchical parent-child chain (SOL→UC→SCEN→SYSREQ→SYSARCH→HWREQ/SWREQ→HWDD/SWDD)
- Acts as a peer/justification link to design artifacts (SYSARCH, SWDD, HWDD)

**Reserved Field Usage**:
- The existing `ItemAttributes.justified_by` field (already reserved in item.rs:420-422) will store ADR links on design artifacts
- ADRs store their linked designs in a new `justifies` field

**New Relationship Types**:
- `Justifies` / `IsJustifiedBy`: Links ADR to design artifacts (bidirectional in graph)
- `Supersedes` / `IsSupersededBy`: Links between ADRs for succession tracking

**Storage Format**:
- Markdown files with YAML frontmatter (consistent with existing items)
- New fields: `status`, `deciders`, `justifies`, `supersedes`, `superseded_by`
- Stored in Git repository alongside other artifacts (Git tracks creation/modification dates)

**Graph Integration**:
- ADR nodes added to `KnowledgeGraph` (petgraph DiGraph)
- `GraphBuilder` extended to create bidirectional justification edges
- `RelationshipRules` extended with ADR-specific validation (ADR can justify SYSARCH, SWDD, HWDD only)

**Key Files to Modify**:
- `sara-core/src/model/item.rs` - Add ItemType::ArchitectureDecisionRecord, ADR-specific attributes
- `sara-core/src/model/relationship.rs` - Add Justifies/Supersedes relationship types
- `sara-core/src/model/field.rs` - Add new FieldName variants for ADR fields
- `sara-core/src/graph/builder.rs` - Extend for ADR relationship building
- `sara-core/src/parser/markdown.rs` - Extend RawFrontmatter for ADR fields
- `sara-core/templates/` - Add ADR template (adr.tera)

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can create a new ADR with all required fields and link it to design artifacts in under 5 minutes.
- **SC-002**: Users can navigate from any ADR to its linked design artifacts and back in under 3 clicks.
- **SC-003**: Users can find any existing ADR using search or filters in under 30 seconds.
- **SC-004**: 100% of ADRs display their current status clearly without requiring additional navigation.
- **SC-005**: All links between ADRs and design artifacts remain intact and navigable after design artifacts are modified (not deleted).
- **SC-006**: Users can trace the complete history of superseded decisions by following supersession links.

## Assumptions

- Design artifacts (SYSARCH, SWDD, HWDD) already exist in the SARA knowledge graph system with established ItemType definitions.
- The existing `ItemAttributes.justified_by` field is available and reserved for ADR linking.
- Users have Git repository access for creating/modifying ADR markdown files.
- The ADR format follows industry-standard structure (title, context, decision, consequences, status) stored as YAML frontmatter.
- A single ADR can link to multiple design artifacts across different types (SYSARCH, SWDD, HWDD) using the Justifies relationship type.
- ADR content is stored as structured text/markdown files consistent with existing Item storage patterns.
- The KnowledgeGraph (petgraph) and GraphBuilder patterns are extended rather than replaced.
