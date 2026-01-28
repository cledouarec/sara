//! Integration tests for the Sara CLI.
//!
//! These tests verify that CLI commands work correctly end-to-end.

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;

/// Get the path to the test fixtures directory (in workspace root).
fn fixtures_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("parent dir")
        .join("tests/fixtures")
}

/// Get the sara command.
fn sara() -> Command {
    Command::new(env!("CARGO_BIN_EXE_sara"))
}

mod parse_command {
    use super::*;

    #[test]
    fn test_parse_valid_graph() {
        let fixtures = fixtures_path().join("valid_graph");

        sara()
            .arg("parse")
            .arg("-r")
            .arg(&fixtures)
            .assert()
            .success()
            .stdout(predicate::str::contains("Parse Results"))
            .stdout(predicate::str::contains("Items parsed:"));
    }

    #[test]
    fn test_parse_nonexistent_repository() {
        // Non-existent repositories are warned about but don't fail the parse
        // They simply result in no items found
        sara()
            .arg("parse")
            .arg("-r")
            .arg("/nonexistent/path")
            .assert()
            .success()
            .stdout(
                predicate::str::contains("No items found")
                    .or(predicate::str::contains("Items parsed: 0")),
            );
    }

    #[test]
    fn test_parse_detects_duplicates() {
        let fixtures = fixtures_path().join("duplicates");

        sara()
            .arg("parse")
            .arg("-r")
            .arg(&fixtures)
            .assert()
            .failure()
            .stdout(predicate::str::contains("Duplicate identifier"));
    }
}

mod validate_command {
    use super::*;

    #[test]
    fn test_validate_valid_graph() {
        let fixtures = fixtures_path().join("valid_graph");

        sara()
            .arg("validate")
            .arg("-r")
            .arg(&fixtures)
            .assert()
            .success()
            .stdout(predicate::str::contains("Validation"));
    }

    #[test]
    fn test_validate_detects_broken_refs() {
        let fixtures = fixtures_path().join("broken_refs");

        // Error details go to stdout, status summary to stderr
        sara()
            .arg("validate")
            .arg("-r")
            .arg(&fixtures)
            .assert()
            .failure()
            .stdout(predicate::str::contains("Broken reference"));
    }

    #[test]
    fn test_validate_detects_duplicates() {
        let fixtures = fixtures_path().join("duplicates");

        // Error details go to stdout, status summary to stderr
        sara()
            .arg("validate")
            .arg("-r")
            .arg(&fixtures)
            .assert()
            .failure()
            .stdout(predicate::str::contains("Duplicate identifier"));
    }

    #[test]
    fn test_validate_strict_mode() {
        let fixtures = fixtures_path().join("orphans");

        // In strict mode, orphans should be errors
        sara()
            .arg("validate")
            .arg("-r")
            .arg(&fixtures)
            .arg("--strict")
            .assert()
            .failure();
    }

    #[test]
    fn test_validate_json_output() {
        let fixtures = fixtures_path().join("valid_graph");

        sara()
            .arg("validate")
            .arg("-r")
            .arg(&fixtures)
            .arg("--format")
            .arg("json")
            .assert()
            .success()
            .stdout(predicate::str::contains("\"items_checked\""));
    }
}

mod query_command {
    use super::*;

    #[test]
    fn test_query_existing_item() {
        let fixtures = fixtures_path().join("valid_graph");

        sara()
            .current_dir(&fixtures)
            .arg("query")
            .arg("SOL-001")
            .assert()
            .success()
            .stdout(predicate::str::contains("SOL-001"))
            .stdout(predicate::str::contains("Customer Portal"));
    }

    #[test]
    fn test_query_nonexistent_item() {
        let fixtures = fixtures_path().join("valid_graph");

        sara()
            .current_dir(&fixtures)
            .arg("query")
            .arg("NONEXISTENT-999")
            .assert()
            .failure()
            .stdout(predicate::str::contains("not found"));
    }

    #[test]
    fn test_query_upstream() {
        let fixtures = fixtures_path().join("valid_graph");

        sara()
            .current_dir(&fixtures)
            .arg("query")
            .arg("SCEN-001")
            .arg("--upstream")
            .assert()
            .success()
            .stdout(predicate::str::contains("SOL-001").or(predicate::str::contains("UC-001")));
    }

    #[test]
    fn test_query_downstream() {
        let fixtures = fixtures_path().join("valid_graph");

        sara()
            .current_dir(&fixtures)
            .arg("query")
            .arg("SOL-001")
            .arg("--downstream")
            .assert()
            .success();
    }
}

mod report_command {
    use super::*;

    #[test]
    fn test_report_coverage() {
        let fixtures = fixtures_path().join("valid_graph");

        sara()
            .current_dir(&fixtures)
            .arg("report")
            .arg("coverage")
            .assert()
            .success()
            .stdout(predicate::str::contains("Coverage"));
    }

    #[test]
    fn test_report_matrix() {
        let fixtures = fixtures_path().join("valid_graph");

        sara()
            .current_dir(&fixtures)
            .arg("report")
            .arg("matrix")
            .assert()
            .success();
    }

    #[test]
    fn test_report_json_format() {
        let fixtures = fixtures_path().join("valid_graph");

        sara()
            .current_dir(&fixtures)
            .arg("report")
            .arg("coverage")
            .arg("--format")
            .arg("json")
            .assert()
            .success()
            .stdout(predicate::str::contains("{"));
    }
}

mod init_command {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_init_new_file() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("TEST-001.md");

        // Create an empty file first
        fs::write(&test_file, "# Test Document\n\nSome content here.").unwrap();

        sara()
            .arg("init")
            .arg("system-requirement")
            .arg(&test_file)
            .assert()
            .success();

        // Verify frontmatter was added
        let content = fs::read_to_string(&test_file).unwrap();
        assert!(content.contains("---"));
        assert!(content.contains("type: system_requirement"));
        assert!(content.contains("id:"));
    }

    #[test]
    fn test_init_with_custom_id() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("MY-REQ.md");

        fs::write(&test_file, "# My Requirement\n").unwrap();

        sara()
            .arg("init")
            .arg("swreq") // Using alias
            .arg(&test_file)
            .arg("--id")
            .arg("SWREQ-999")
            .assert()
            .success();

        let content = fs::read_to_string(&test_file).unwrap();
        assert!(content.contains("id: \"SWREQ-999\"") || content.contains("id: SWREQ-999"));
    }

    #[test]
    fn test_init_requires_type() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("NO-TYPE.md");

        fs::write(&test_file, "# No Type\n").unwrap();

        // Without a subcommand, interactive mode fails without TTY
        sara().arg("init").assert().failure();
    }

    #[test]
    fn test_init_existing_frontmatter_without_force() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("EXISTING.md");

        // Create file with existing frontmatter
        fs::write(
            &test_file,
            "---\nid: EXISTING-001\ntype: solution\nname: Existing\n---\n# Content\n",
        )
        .unwrap();

        // Should fail without --force
        sara()
            .arg("init")
            .arg("use-case")
            .arg(&test_file)
            .assert()
            .failure();
    }

    #[test]
    fn test_init_existing_frontmatter_with_force() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("OVERWRITE.md");

        // Create file with existing frontmatter
        fs::write(
            &test_file,
            "---\nid: OLD-001\ntype: solution\nname: Old\n---\n# Content\n",
        )
        .unwrap();

        sara()
            .arg("init")
            .arg("uc") // Using alias
            .arg(&test_file)
            .arg("--force")
            .assert()
            .success();

        let content = fs::read_to_string(&test_file).unwrap();
        assert!(content.contains("type: use_case"));
    }

    #[test]
    fn test_init_invalid_type() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("BAD-TYPE.md");

        fs::write(&test_file, "# Bad Type\n").unwrap();

        // Invalid subcommand should fail
        sara()
            .arg("init")
            .arg("invalid_type")
            .arg(&test_file)
            .assert()
            .failure();
    }
}

mod diff_command {
    use super::*;

    // Note: diff command requires a git repository context.
    // These tests verify basic argument parsing and help text.

    #[test]
    fn test_diff_help() {
        sara()
            .arg("diff")
            .arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains("Compare"))
            .stdout(predicate::str::contains("REF1"))
            .stdout(predicate::str::contains("REF2"));
    }

    #[test]
    fn test_diff_requires_two_refs() {
        sara().arg("diff").arg("main").assert().failure();
    }
}

mod cycles_detection {
    use super::*;

    #[test]
    fn test_validate_detects_cycles() {
        let fixtures = fixtures_path().join("cycles");

        // Error details go to stdout, status summary to stderr
        sara()
            .arg("validate")
            .arg("-r")
            .arg(&fixtures)
            .assert()
            .failure()
            .stdout(predicate::str::contains("Circular").or(predicate::str::contains("cycle")));
    }
}

mod csv_output {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_report_csv_format() {
        let fixtures = fixtures_path().join("valid_graph");

        sara()
            .current_dir(&fixtures)
            .arg("report")
            .arg("matrix")
            .arg("--format")
            .arg("csv")
            .assert()
            .success()
            .stdout(predicate::str::contains(","));
    }

    #[test]
    fn test_report_coverage_csv() {
        let fixtures = fixtures_path().join("valid_graph");

        sara()
            .current_dir(&fixtures)
            .arg("report")
            .arg("coverage")
            .arg("--format")
            .arg("csv")
            .assert()
            .success();
    }

    #[test]
    fn test_report_output_to_file() {
        let fixtures = fixtures_path().join("valid_graph");
        let temp_dir = TempDir::new().unwrap();
        let output_file = temp_dir.path().join("report.csv");

        sara()
            .current_dir(&fixtures)
            .arg("report")
            .arg("matrix")
            .arg("--format")
            .arg("csv")
            .arg("-o")
            .arg(&output_file)
            .assert()
            .success();

        assert!(output_file.exists());
    }
}

mod query_formats {
    use super::*;

    #[test]
    fn test_query_json_format() {
        let fixtures = fixtures_path().join("valid_graph");

        sara()
            .current_dir(&fixtures)
            .arg("query")
            .arg("SOL-001")
            .arg("--format")
            .arg("json")
            .arg("--downstream")
            .assert()
            .success()
            .stdout(predicate::str::contains("{"));
    }

    #[test]
    fn test_query_with_depth_limit() {
        let fixtures = fixtures_path().join("valid_graph");

        sara()
            .current_dir(&fixtures)
            .arg("query")
            .arg("SOL-001")
            .arg("--downstream")
            .arg("--depth")
            .arg("1")
            .assert()
            .success();
    }

    #[test]
    fn test_query_with_type_filter() {
        let fixtures = fixtures_path().join("valid_graph");

        sara()
            .current_dir(&fixtures)
            .arg("query")
            .arg("SOL-001")
            .arg("--downstream")
            .arg("--type")
            .arg("use_case")
            .assert()
            .success();
    }
}

mod interactive_mode {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_init_without_subcommand_fails_in_non_tty() {
        // When no subcommand is provided, enters interactive mode which fails without TTY
        sara()
            .arg("init")
            .assert()
            .failure()
            .stdout(predicate::str::contains("terminal").or(predicate::str::contains("TTY")));
    }

    #[test]
    fn test_init_help_shows_subcommands() {
        // The help text should show available subcommands
        sara()
            .arg("init")
            .arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains("interactive mode"))
            .stdout(predicate::str::contains("adr"))
            .stdout(predicate::str::contains("solution"))
            .stdout(predicate::str::contains("system-requirement"));
    }

    #[test]
    fn test_init_with_subcommand_bypasses_interactive() {
        // When subcommand is provided, should work without TTY
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("NONINTERACTIVE.md");
        fs::write(&test_file, "# Non-Interactive Test\n").unwrap();

        sara()
            .arg("init")
            .arg("solution")
            .arg(&test_file)
            .assert()
            .success();

        let content = fs::read_to_string(&test_file).unwrap();
        assert!(content.contains("type: solution"));
    }

    #[test]
    fn test_init_without_subcommand_in_non_interactive_fails() {
        // When no subcommand, enters interactive mode which fails without TTY
        sara().arg("init").assert().failure();
    }

    #[test]
    fn test_init_prefilled_args_with_subcommand() {
        // Prefilled arguments should work with subcommand (non-interactive)
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("PREFILLED.md");
        fs::write(&test_file, "# Prefilled Test\n").unwrap();

        sara()
            .arg("init")
            .arg("use-case")
            .arg(&test_file)
            .arg("--id")
            .arg("UC-CUSTOM-001")
            .arg("--name")
            .arg("Custom Use Case")
            .arg("--description")
            .arg("A custom description")
            .assert()
            .success();

        let content = fs::read_to_string(&test_file).unwrap();
        assert!(content.contains("id:"));
        assert!(content.contains("UC-CUSTOM-001"));
        assert!(content.contains("Custom Use Case"));
    }
}

mod edit_command {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_edit_help() {
        sara()
            .arg("edit")
            .arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains("Edit"))
            .stdout(predicate::str::contains("ITEM_ID"))
            .stdout(predicate::str::contains("--name"))
            .stdout(predicate::str::contains("--description"));
    }

    #[test]
    fn test_edit_help_shows_headings() {
        sara()
            .arg("edit")
            .arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains("Item Properties"))
            .stdout(predicate::str::contains("Traceability"))
            .stdout(predicate::str::contains("Type-Specific"));
    }

    #[test]
    fn test_edit_item_not_found() {
        let fixtures = fixtures_path().join("edit_tests");

        sara()
            .current_dir(&fixtures)
            .arg("edit")
            .arg("NONEXISTENT-999")
            .arg("--name")
            .arg("New Name")
            .assert()
            .failure()
            .stdout(predicate::str::contains("not found"));
    }

    #[test]
    fn test_edit_item_not_found_with_suggestions() {
        let fixtures = fixtures_path().join("edit_tests");

        // Query for SYSREQ-EDIT-01 (missing digit) should suggest SYSREQ-EDIT-001
        // Suggestions are printed to stdout
        sara()
            .current_dir(&fixtures)
            .arg("edit")
            .arg("SYSREQ-EDIT-01")
            .arg("--name")
            .arg("New Name")
            .assert()
            .failure()
            .stdout(predicate::str::contains("Did you mean"));
    }

    #[test]
    fn test_edit_non_interactive_name_change() {
        let fixtures = fixtures_path().join("edit_tests");
        let temp_dir = TempDir::new().unwrap();

        // Copy fixture to temp location
        let original = fixtures.join("SYSREQ-EDIT-001.md");
        let test_file = temp_dir.path().join("SYSREQ-EDIT-001.md");
        fs::copy(&original, &test_file).unwrap();

        sara()
            .current_dir(temp_dir.path())
            .arg("edit")
            .arg("SYSREQ-EDIT-001")
            .arg("--name")
            .arg("Updated Requirement Name")
            .assert()
            .success()
            .stdout(predicate::str::contains("Updated").or(predicate::str::contains("modified")));

        // Verify the file was updated
        let content = fs::read_to_string(&test_file).unwrap();
        assert!(content.contains("Updated Requirement Name"));
    }

    #[test]
    fn test_edit_non_interactive_specification_change() {
        let fixtures = fixtures_path().join("edit_tests");
        let temp_dir = TempDir::new().unwrap();

        // Copy fixture to temp location
        let original = fixtures.join("SYSREQ-EDIT-001.md");
        let test_file = temp_dir.path().join("SYSREQ-EDIT-001.md");
        fs::copy(&original, &test_file).unwrap();

        sara()
            .current_dir(temp_dir.path())
            .arg("edit")
            .arg("SYSREQ-EDIT-001")
            .arg("--specification")
            .arg("The system SHALL now do something different.")
            .assert()
            .success();

        // Verify the file was updated
        let content = fs::read_to_string(&test_file).unwrap();
        assert!(content.contains("The system SHALL now do something different."));
    }

    #[test]
    fn test_edit_without_flags_requires_tty() {
        let fixtures = fixtures_path().join("edit_tests");

        // Running without any modification flags triggers interactive mode
        // which fails without TTY
        sara()
            .current_dir(&fixtures)
            .arg("edit")
            .arg("SYSREQ-EDIT-001")
            .assert()
            .failure()
            .stdout(predicate::str::contains("terminal").or(predicate::str::contains("TTY")));
    }

    #[test]
    fn test_edit_invalid_type_specific_option() {
        let fixtures = fixtures_path().join("edit_tests");
        let temp_dir = TempDir::new().unwrap();

        // Copy solution fixture to temp location
        let original = fixtures.join("SOL-EDIT-001.md");
        let test_file = temp_dir.path().join("SOL-EDIT-001.md");
        fs::copy(&original, &test_file).unwrap();

        // --specification is only valid for requirements, not solutions
        sara()
            .current_dir(temp_dir.path())
            .arg("edit")
            .arg("SOL-EDIT-001")
            .arg("--specification")
            .arg("Invalid specification for solution")
            .assert()
            .failure()
            .stdout(predicate::str::contains("only valid for requirement"));
    }

    #[test]
    fn test_edit_preserves_body_content() {
        let fixtures = fixtures_path().join("edit_tests");
        let temp_dir = TempDir::new().unwrap();

        // Copy fixture to temp location
        let original = fixtures.join("SYSREQ-EDIT-001.md");
        let test_file = temp_dir.path().join("SYSREQ-EDIT-001.md");
        fs::copy(&original, &test_file).unwrap();

        // Read original body content
        let original_content = fs::read_to_string(&original).unwrap();
        assert!(original_content.contains("Rationale"));

        sara()
            .current_dir(temp_dir.path())
            .arg("edit")
            .arg("SYSREQ-EDIT-001")
            .arg("--name")
            .arg("Changed Name")
            .assert()
            .success();

        // Verify body content is preserved
        let new_content = fs::read_to_string(&test_file).unwrap();
        assert!(new_content.contains("Rationale"));
        assert!(new_content.contains("Changed Name"));
    }

    #[test]
    fn test_edit_multiple_fields() {
        let fixtures = fixtures_path().join("edit_tests");
        let temp_dir = TempDir::new().unwrap();

        // Copy fixture to temp location
        let original = fixtures.join("SYSREQ-EDIT-001.md");
        let test_file = temp_dir.path().join("SYSREQ-EDIT-001.md");
        fs::copy(&original, &test_file).unwrap();

        sara()
            .current_dir(temp_dir.path())
            .arg("edit")
            .arg("SYSREQ-EDIT-001")
            .arg("--name")
            .arg("Multi-Field Update")
            .arg("--description")
            .arg("Updated description text")
            .assert()
            .success();

        // Verify both fields were updated
        let content = fs::read_to_string(&test_file).unwrap();
        assert!(content.contains("Multi-Field Update"));
        assert!(content.contains("Updated description text"));
    }
}

mod global_options {
    use super::*;

    #[test]
    fn test_help() {
        sara()
            .arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains("sara"))
            .stdout(predicate::str::contains("parse"))
            .stdout(predicate::str::contains("validate"))
            .stdout(predicate::str::contains("query"))
            .stdout(predicate::str::contains("report"));
    }

    #[test]
    fn test_version() {
        sara()
            .arg("--version")
            .assert()
            .success()
            .stdout(predicate::str::contains("sara"));
    }

    #[test]
    fn test_no_color_flag() {
        let fixtures = fixtures_path().join("valid_graph");

        sara()
            .arg("--no-color")
            .arg("parse")
            .arg("-r")
            .arg(&fixtures)
            .assert()
            .success();
    }

    #[test]
    fn test_no_emoji_flag() {
        let fixtures = fixtures_path().join("valid_graph");

        sara()
            .arg("--no-emoji")
            .arg("parse")
            .arg("-r")
            .arg(&fixtures)
            .assert()
            .success();
    }

    #[test]
    fn test_no_color_env_var() {
        let fixtures = fixtures_path().join("valid_graph");

        sara()
            .env("NO_COLOR", "1")
            .arg("parse")
            .arg("-r")
            .arg(&fixtures)
            .assert()
            .success();
    }
}
