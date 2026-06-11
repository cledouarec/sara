//! Integration tests for the Sara CLI.
//!
//! These tests verify that CLI commands work correctly end-to-end.

use std::path::PathBuf;

use assert_cmd::Command;
use predicates::prelude::*;

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

mod check_command {
    use super::*;

    #[test]
    fn test_check_valid_graph() {
        let fixtures = fixtures_path().join("valid_graph");

        sara()
            .arg("check")
            .arg("-r")
            .arg(&fixtures)
            .assert()
            .success()
            .stdout(predicate::str::contains("Check Results"))
            .stdout(predicate::str::contains("Items:"));
    }

    #[test]
    fn test_check_nonexistent_repository() {
        // Non-existent repositories are warned about but don't fail the check
        // They simply result in no items found
        sara()
            .arg("check")
            .arg("-r")
            .arg("/nonexistent/path")
            .assert()
            .success()
            .stdout(
                predicate::str::contains("No items found").or(predicate::str::contains("Items: 0")),
            );
    }

    #[test]
    fn test_check_detects_duplicates() {
        let fixtures = fixtures_path().join("duplicates");

        sara()
            .arg("check")
            .arg("-r")
            .arg(&fixtures)
            .assert()
            .failure()
            .stdout(predicate::str::contains("Duplicate identifier"));
    }

    #[test]
    fn test_check_detects_broken_refs() {
        let fixtures = fixtures_path().join("broken_refs");

        // Error details go to stdout, status summary to stderr
        sara()
            .arg("check")
            .arg("-r")
            .arg(&fixtures)
            .assert()
            .failure()
            .stdout(predicate::str::contains("Broken reference"));
    }

    #[test]
    fn test_check_strict_mode() {
        let fixtures = fixtures_path().join("orphans");

        // In strict mode, orphans should be errors
        sara()
            .arg("check")
            .arg("-r")
            .arg(&fixtures)
            .arg("--strict")
            .assert()
            .failure();
    }

    #[test]
    fn test_check_json_output() {
        let fixtures = fixtures_path().join("valid_graph");

        sara()
            .arg("check")
            .arg("-r")
            .arg(&fixtures)
            .arg("--format")
            .arg("json")
            .assert()
            .success()
            .stdout(predicate::str::contains("\"items_checked\""));
    }

    #[test]
    fn test_check_warns_about_skipped_files() {
        let fixtures = fixtures_path().join("corrupt_file");

        // The corrupt file is reported but does not fail the check
        sara()
            .arg("check")
            .arg("-r")
            .arg(&fixtures)
            .assert()
            .success()
            .stdout(predicate::str::contains("skipped"))
            .stdout(predicate::str::contains("CORRUPT-001.md"))
            .stdout(predicate::str::contains("Check passed"));
    }

    #[test]
    fn test_check_strict_config_fails_on_skipped_files() {
        let fixtures = fixtures_path().join("corrupt_file");
        let temp_dir = tempfile::TempDir::new().unwrap();
        let config_path = temp_dir.path().join("sara.toml");
        std::fs::write(&config_path, "[validation]\nstrict_mode = true\n").unwrap();

        sara()
            .arg("--config")
            .arg(&config_path)
            .arg("check")
            .arg("-r")
            .arg(&fixtures)
            .assert()
            .failure()
            .stdout(predicate::str::contains("CORRUPT-001.md"))
            .stdout(predicate::str::contains("skipped during scan"));
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
    use std::fs;

    use tempfile::TempDir;

    use super::*;

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
    use std::fs;
    use std::path::Path;
    use std::process::Command as SystemCommand;

    use tempfile::TempDir;

    use super::*;

    /// Item renamed by the second commit, as committed in the baseline.
    const RENAMED_ITEM_BEFORE: &str = r#"---
id: "SOL-001"
type: solution
name: "Payment Service"
---
# Solution: Payment Service
"#;

    /// Item renamed by the second commit, after the rename.
    const RENAMED_ITEM_AFTER: &str = r#"---
id: "SOL-001"
type: solution
name: "Payment Platform"
---
# Solution: Payment Platform
"#;

    /// Item present in the baseline and deleted by the second commit.
    const REMOVED_ITEM: &str = r#"---
id: "SOL-002"
type: solution
name: "Legacy Gateway"
---
# Solution: Legacy Gateway
"#;

    /// Item introduced by the second commit.
    const ADDED_ITEM: &str = r#"---
id: "SOL-003"
type: solution
name: "Mobile App"
---
# Solution: Mobile App
"#;

    /// Valid item outside the configured repository path, present in both
    /// commits.
    const OUT_OF_SCOPE_ITEM: &str = r#"---
id: "SOL-099"
type: solution
name: "Out of Scope"
---
# Solution: Out of Scope
"#;

    /// File with invalid frontmatter outside the configured repository path,
    /// present in both commits.
    const OUT_OF_SCOPE_CORRUPT_ITEM: &str = r#"---
id: "CORRUPT-001
type: solution
---
# Corrupt
"#;

    /// Item added under the second configured repository path by the second
    /// commit.
    const SECOND_PATH_ADDED_ITEM: &str = r#"---
id: "SOL-010"
type: solution
name: "Reporting Service"
---
# Solution: Reporting Service
"#;

    /// Runs a Git command in the repository, isolated from the user and
    /// system Git configuration.
    fn git(repo: &Path, args: &[&str]) {
        let output = SystemCommand::new("git")
            .arg("-C")
            .arg(repo)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_SYSTEM", "/dev/null")
            .args(args)
            .output()
            .expect("failed to run git");
        assert!(
            output.status.success(),
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    /// Creates a Git repository whose `baseline` branch holds SOL-001 and
    /// SOL-002, while the second commit at HEAD renames SOL-001, removes
    /// SOL-002 and adds SOL-003.
    ///
    /// The configuration declares `./docs` as the only repository path; a
    /// valid item and a corrupt file live outside it in both commits and
    /// must never appear in any output.
    fn diff_repo() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let repo = temp_dir.path();

        git(repo, &["init"]);
        git(repo, &["config", "user.name", "Sara Tests"]);
        git(repo, &["config", "user.email", "tests@example.com"]);

        fs::write(
            repo.join("sara.toml"),
            "[repositories]\npaths = [\"./docs\"]\n",
        )
        .unwrap();
        fs::create_dir(repo.join("docs")).unwrap();
        fs::write(repo.join("docs/SOL-001.md"), RENAMED_ITEM_BEFORE).unwrap();
        fs::write(repo.join("docs/SOL-002.md"), REMOVED_ITEM).unwrap();
        fs::write(repo.join("SOL-099.md"), OUT_OF_SCOPE_ITEM).unwrap();
        fs::write(repo.join("CORRUPT-001.md"), OUT_OF_SCOPE_CORRUPT_ITEM).unwrap();
        git(repo, &["add", "."]);
        git(repo, &["commit", "-m", "baseline"]);
        git(repo, &["branch", "baseline"]);

        fs::write(repo.join("docs/SOL-001.md"), RENAMED_ITEM_AFTER).unwrap();
        fs::write(repo.join("docs/SOL-003.md"), ADDED_ITEM).unwrap();
        fs::remove_file(repo.join("docs/SOL-002.md")).unwrap();
        git(repo, &["add", "."]);
        git(repo, &["commit", "-m", "changes"]);

        temp_dir
    }

    /// Creates a Git repository whose configuration declares `./docs` and
    /// `./specs` as repository paths. The baseline commit holds SOL-001
    /// under `docs`; the second commit adds SOL-010 under `specs`.
    fn multi_path_repo() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let repo = temp_dir.path();

        git(repo, &["init"]);
        git(repo, &["config", "user.name", "Sara Tests"]);
        git(repo, &["config", "user.email", "tests@example.com"]);

        fs::write(
            repo.join("sara.toml"),
            "[repositories]\npaths = [\"./docs\", \"./specs\"]\n",
        )
        .unwrap();
        fs::create_dir(repo.join("docs")).unwrap();
        fs::write(repo.join("docs/SOL-001.md"), RENAMED_ITEM_BEFORE).unwrap();
        git(repo, &["add", "."]);
        git(repo, &["commit", "-m", "baseline"]);

        fs::create_dir(repo.join("specs")).unwrap();
        fs::write(repo.join("specs/SOL-010.md"), SECOND_PATH_ADDED_ITEM).unwrap();
        git(repo, &["add", "."]);
        git(repo, &["commit", "-m", "add spec"]);

        temp_dir
    }

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

    #[test]
    fn test_diff_text_lists_added_removed_modified() {
        let repo = diff_repo();

        sara()
            .current_dir(repo.path())
            .arg("--no-color")
            .arg("diff")
            .arg("baseline")
            .arg("HEAD")
            .assert()
            .success()
            .stdout(predicate::str::contains("not fully implemented").not())
            .stdout(predicate::str::contains("Added Items:"))
            .stdout(predicate::str::contains("+ SOL-003 (Solution)"))
            .stdout(predicate::str::contains("Removed Items:"))
            .stdout(predicate::str::contains("- SOL-002 (Solution)"))
            .stdout(predicate::str::contains("Modified Items:"))
            .stdout(predicate::str::contains("~ SOL-001 (Solution)"))
            .stdout(predicate::str::contains(
                "name: Payment Service → Payment Platform",
            ));
    }

    #[test]
    fn test_diff_revision_expression_lists_changes() {
        let repo = diff_repo();

        sara()
            .current_dir(repo.path())
            .arg("--no-color")
            .arg("diff")
            .arg("HEAD~1")
            .arg("HEAD")
            .assert()
            .success()
            .stdout(predicate::str::contains("+ SOL-003 (Solution)"))
            .stdout(predicate::str::contains("- SOL-002 (Solution)"))
            .stdout(predicate::str::contains("~ SOL-001 (Solution)"));
    }

    #[test]
    fn test_diff_stat_prints_summary_only() {
        let repo = diff_repo();

        sara()
            .current_dir(repo.path())
            .arg("--no-color")
            .arg("diff")
            .arg("baseline")
            .arg("HEAD")
            .arg("--stat")
            .assert()
            .success()
            .stdout(predicate::str::contains("Items:         +1 -1 ~1"))
            .stdout(predicate::str::contains("Relationships: +0 -0"))
            .stdout(predicate::str::contains("Added Items:").not());
    }

    #[test]
    fn test_diff_json_output() {
        let repo = diff_repo();

        let assert = sara()
            .current_dir(repo.path())
            .arg("diff")
            .arg("baseline")
            .arg("HEAD")
            .arg("--format")
            .arg("json")
            .assert()
            .success();

        let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
        let diff: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON output");

        assert_eq!(diff["added_items"][0]["id"], "SOL-003");
        assert_eq!(diff["removed_items"][0]["id"], "SOL-002");
        assert_eq!(diff["modified_items"][0]["id"], "SOL-001");
        assert_eq!(diff["modified_items"][0]["changes"][0]["field"], "name");
        assert_eq!(
            diff["modified_items"][0]["changes"][0]["old_value"],
            "Payment Service"
        );
        assert_eq!(
            diff["modified_items"][0]["changes"][0]["new_value"],
            "Payment Platform"
        );
        assert_eq!(diff["stats"]["items_added"], 1);
        assert_eq!(diff["stats"]["items_removed"], 1);
        assert_eq!(diff["stats"]["items_modified"], 1);
    }

    #[test]
    fn test_diff_same_ref_reports_no_changes() {
        let repo = diff_repo();

        sara()
            .current_dir(repo.path())
            .arg("diff")
            .arg("HEAD")
            .arg("HEAD")
            .assert()
            .success()
            .stdout(predicate::str::contains("No changes detected"));
    }

    #[test]
    fn test_diff_unknown_ref_fails() {
        let repo = diff_repo();

        sara()
            .current_dir(repo.path())
            .arg("diff")
            .arg("no-such-branch")
            .arg("HEAD")
            .assert()
            .failure()
            .stdout(predicate::str::contains("Failed to parse repository"))
            .stdout(predicate::str::contains("no-such-branch"));
    }

    #[test]
    fn test_diff_ignores_files_outside_configured_repositories() {
        let repo = diff_repo();

        sara()
            .current_dir(repo.path())
            .arg("--no-color")
            .arg("diff")
            .arg("baseline")
            .arg("HEAD")
            .assert()
            .success()
            .stdout(predicate::str::contains("~ SOL-001 (Solution)"))
            .stdout(predicate::str::contains("SOL-099").not())
            .stdout(predicate::str::contains("CORRUPT-001").not())
            .stderr(predicate::str::contains("CORRUPT-001").not());
    }

    #[test]
    fn test_diff_scans_all_configured_repository_paths() {
        let repo = multi_path_repo();

        sara()
            .current_dir(repo.path())
            .arg("--no-color")
            .arg("diff")
            .arg("HEAD~1")
            .arg("HEAD")
            .assert()
            .success()
            .stdout(predicate::str::contains("+ SOL-010 (Solution)"));
    }

    #[test]
    fn test_check_at_ignores_files_outside_configured_repositories() {
        let repo = diff_repo();

        sara()
            .current_dir(repo.path())
            .arg("--no-color")
            .arg("check")
            .arg("--at")
            .arg("HEAD")
            .assert()
            .success()
            .stdout(predicate::str::contains("SOL-099").not())
            .stdout(predicate::str::contains("CORRUPT-001").not())
            .stderr(predicate::str::contains("CORRUPT-001").not());
    }
}

mod cycles_detection {
    use super::*;

    #[test]
    fn test_check_detects_cycles() {
        let fixtures = fixtures_path().join("cycles");

        // Error details go to stdout, status summary to stderr
        sara()
            .arg("check")
            .arg("-r")
            .arg(&fixtures)
            .assert()
            .failure()
            .stdout(predicate::str::contains("Circular").or(predicate::str::contains("cycle")));
    }
}

mod csv_output {
    use tempfile::TempDir;

    use super::*;

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
    use std::fs;

    use tempfile::TempDir;

    use super::*;

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
    use std::fs;

    use tempfile::TempDir;

    use super::*;

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

        // Solution declares no specification field
        sara()
            .current_dir(temp_dir.path())
            .arg("edit")
            .arg("SOL-EDIT-001")
            .arg("--specification")
            .arg("Invalid specification for solution")
            .assert()
            .failure()
            .stdout(predicate::str::contains("not a field declared by Solution"));
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
            .stdout(predicate::str::contains("check"))
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
            .arg("check")
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
            .arg("check")
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
            .arg("check")
            .arg("-r")
            .arg(&fixtures)
            .assert()
            .success();
    }
}

mod custom_schema {
    use std::fs;
    use std::path::Path;

    use tempfile::TempDir;

    use super::*;

    /// Renders the `model_schema` config line for a sara.toml.
    ///
    /// The path is written with forward slashes: backslashes in Windows
    /// temp-dir paths would start escape sequences inside a TOML basic
    /// string and break parsing.
    fn model_schema_config(schema_path: &Path) -> String {
        format!(
            "model_schema = \"{}\"\n",
            schema_path.display().to_string().replace('\\', "/")
        )
    }

    /// YAML appended to an exported built-in model to declare a new type.
    const CUSTOM_TYPE_YAML: &str = r#"- id: stakeholder_requirement
  display_name: Stakeholder Requirement
  prefix: STKREQ
  id_format: "{prefix}-{seq:03}"
  parent_types:
  - solution
  fields:
  - name: rationale
    display_name: Rationale
    field_type: text
    required: true
    placeholder: TBD
  allowed_targets:
  - relation: refines
    targets:
    - solution
"#;

    /// Exports the built-in model, appends a custom type and returns the
    /// config path referencing the extended schema.
    fn write_extended_schema(temp_dir: &TempDir) -> std::path::PathBuf {
        let schema_path = temp_dir.path().join("model.yaml");
        sara()
            .arg("schema")
            .arg("-o")
            .arg(&schema_path)
            .assert()
            .success();

        let exported = fs::read_to_string(&schema_path).unwrap();
        let extended = exported.replace("\nrelations:", &format!("\n{CUSTOM_TYPE_YAML}relations:"));
        fs::write(&schema_path, extended).unwrap();

        let config_path = temp_dir.path().join("sara.toml");
        fs::write(&config_path, model_schema_config(&schema_path)).unwrap();
        config_path
    }

    #[test]
    fn test_schema_exports_builtin_model() {
        sara()
            .arg("schema")
            .assert()
            .success()
            .stdout(predicate::str::contains("item_types:"))
            .stdout(predicate::str::contains("- id: solution"))
            .stdout(predicate::str::contains("relations:"));
    }

    #[test]
    fn test_schema_exports_the_configured_model() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = write_extended_schema(&temp_dir);

        sara()
            .arg("--config")
            .arg(&config_path)
            .arg("schema")
            .assert()
            .success()
            .stdout(predicate::str::contains("stakeholder_requirement"));

        // --builtin ignores the configured model.
        sara()
            .arg("--config")
            .arg(&config_path)
            .arg("schema")
            .arg("--builtin")
            .assert()
            .success()
            .stdout(predicate::str::contains("stakeholder_requirement").not());
    }

    #[test]
    fn test_init_subcommand_for_custom_type() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = write_extended_schema(&temp_dir);
        let test_file = temp_dir.path().join("STKREQ-001.md");

        sara()
            .arg("--config")
            .arg(&config_path)
            .arg("init")
            .arg("stakeholder-requirement")
            .arg(&test_file)
            .arg("--id")
            .arg("STKREQ-001")
            .arg("--name")
            .arg("Operator overview")
            .arg("--rationale")
            .arg("Operators need a single pane of glass.")
            .assert()
            .success();

        let content = fs::read_to_string(&test_file).unwrap();
        assert!(content.contains("type: stakeholder_requirement"));
        assert!(content.contains("id: \"STKREQ-001\""));
        assert!(content.contains("rationale: \"Operators need a single pane of glass.\""));
        assert!(content.contains("# Stakeholder Requirement: Operator overview"));
    }

    #[test]
    fn test_init_prefix_alias_for_custom_type() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = write_extended_schema(&temp_dir);
        let test_file = temp_dir.path().join("STKREQ-002.md");

        sara()
            .arg("--config")
            .arg(&config_path)
            .arg("init")
            .arg("stkreq")
            .arg(&test_file)
            .assert()
            .success();

        let content = fs::read_to_string(&test_file).unwrap();
        assert!(content.contains("type: stakeholder_requirement"));
        assert!(content.contains("rationale: \"TBD\""));
    }

    /// YAML appended to an exported built-in model to declare a type linked
    /// through a custom relation pair.
    const CUSTOM_RELATION_TYPE_YAML: &str = r#"- id: test_case
  display_name: Test Case
  prefix: TC
  id_format: "{prefix}-{seq:03}"
  parent_types:
  - system_requirement
  fields: []
  allowed_targets:
  - relation: verifies
    targets:
    - system_requirement
"#;

    /// YAML appended to the exported relations to declare the custom pair.
    const CUSTOM_RELATION_YAML: &str = r#"- id: verifies
  display_name: Verifies
  inverse: is_verified_by
  direction: upstream
  primary: true
- id: is_verified_by
  display_name: Is verified by
  inverse: verifies
  direction: downstream
  primary: false
"#;

    /// Exports the built-in model, appends a custom relation and its type,
    /// and returns the config path referencing the extended schema.
    fn write_schema_with_custom_relation(temp_dir: &TempDir) -> std::path::PathBuf {
        let schema_path = temp_dir.path().join("model.yaml");
        sara()
            .arg("schema")
            .arg("-o")
            .arg(&schema_path)
            .assert()
            .success();

        let exported = fs::read_to_string(&schema_path).unwrap();
        let mut extended = exported.replace(
            "\nrelations:",
            &format!("\n{CUSTOM_RELATION_TYPE_YAML}relations:"),
        );
        extended.push_str(CUSTOM_RELATION_YAML);
        fs::write(&schema_path, extended).unwrap();

        let config_path = temp_dir.path().join("sara.toml");
        fs::write(&config_path, model_schema_config(&schema_path)).unwrap();
        config_path
    }

    #[test]
    fn test_edit_flag_for_custom_relation() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = write_schema_with_custom_relation(&temp_dir);

        // The relation appears as an edit flag.
        sara()
            .arg("--config")
            .arg(&config_path)
            .arg("edit")
            .arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains("--verifies"));

        fs::write(
            temp_dir.path().join("TC-001.md"),
            "---\nid: \"TC-001\"\ntype: test_case\nname: \"Latency check\"\nverifies:\n  - \"SYSREQ-001\"\n---\n# Test Case: Latency check\n",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("SYSREQ-001.md"),
            "---\nid: \"SYSREQ-001\"\ntype: system_requirement\nname: \"Latency budget\"\nspecification: \"The system SHALL respond within 200ms.\"\n---\n",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("SYSREQ-002.md"),
            "---\nid: \"SYSREQ-002\"\ntype: system_requirement\nname: \"Throughput budget\"\nspecification: \"The system SHALL sustain 100 rps.\"\n---\n",
        )
        .unwrap();

        sara()
            .current_dir(temp_dir.path())
            .arg("--config")
            .arg(&config_path)
            .arg("edit")
            .arg("TC-001")
            .arg("--verifies")
            .arg("SYSREQ-002")
            .assert()
            .success();

        let content = fs::read_to_string(temp_dir.path().join("TC-001.md")).unwrap();
        assert!(content.contains("verifies:"));
        assert!(content.contains("- \"SYSREQ-002\""));
        assert!(!content.contains("SYSREQ-001"));
    }

    #[test]
    fn test_custom_schema_replaces_the_builtin_model() {
        let temp_dir = TempDir::new().unwrap();
        let schema_path = temp_dir.path().join("model.yaml");
        fs::write(
            &schema_path,
            r#"item_types:
- id: note
  display_name: Note
  prefix: NOTE
  id_format: "{prefix}-{seq:03}"
  parent_types: []
  fields: []
  allowed_targets: []
relations: []
"#,
        )
        .unwrap();
        let config_path = temp_dir.path().join("sara.toml");
        fs::write(&config_path, model_schema_config(&schema_path)).unwrap();

        // The custom type exists...
        let note_file = temp_dir.path().join("NOTE-001.md");
        sara()
            .arg("--config")
            .arg(&config_path)
            .arg("init")
            .arg("note")
            .arg(&note_file)
            .assert()
            .success();

        // ...and the built-in types are gone.
        sara()
            .arg("--config")
            .arg(&config_path)
            .arg("init")
            .arg("solution")
            .arg(temp_dir.path().join("SOL-001.md"))
            .assert()
            .failure();
    }
}
