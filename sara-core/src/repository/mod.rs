//! Repository scanning and file operations.

pub mod git;
mod scanner;

pub use git::{GitReader, GitRef, get_repo_root, is_git_repo};
pub use scanner::{parse_directory, parse_repositories, scan_directory};
