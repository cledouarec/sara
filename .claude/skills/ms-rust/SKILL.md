---
name: ms-rust
description: ALWAYS use this skill BEFORE writing or modifying ANY Rust code (.rs files), even for simple Hello World programs. Enforces Microsoft Rust coding guidelines, applies M-CANONICAL-DOCS documentation, adds compliance comments, and validates against rust-guidelines.txt. This skill is MANDATORY for all Rust development.
---

# Rust Development
This skill automatically enforces Rust coding standards and best practices when creating or modifying Rust code.

## Instructions

**CRITICAL**: This skill MUST be invoked for ANY Rust code operation, including:
- Creating new .rs files (even simple examples like Hello World)
- Modifying existing .rs files (any change, no matter how small)
- Reviewing Rust code
- Refactoring Rust code

**Process**:
1. Read the [rust-guidelines.txt](rust-guidelines.txt) to understand all compliance requirements
2. Before writing/modifying ANY Rust code, ensure edits are conformant to the guidelines
3. Apply proper M-CANONICAL-DOCS documentation format
4. Add compliance comments
5. Comments must ALWAYS be written in American English, unless the user explicitly requests ‘write comments in French’ or provides another clear instruction specifying a different comment language.
6. If the file is fully compliant, add a comment: `// Rust guideline compliant {date}` where {date} is the guideline date/version

**No exceptions**: Even for trivial code like "Hello World", guidelines must be followed.

---