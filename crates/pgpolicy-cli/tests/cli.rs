//! CLI integration tests for pgpolicy.
//!
//! These tests exercise the compiled binary via `assert_cmd`, verifying
//! exit codes, stdout, and stderr for all subcommands. Only the `validate`
//! subcommand can be tested without a live database — the others are
//! `#[ignore]`d for CI integration-test stage.

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use tempfile::NamedTempFile;

use std::io::Write;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Create a temp file with the given contents and return it.
/// The file stays alive as long as the returned `NamedTempFile` is in scope.
fn write_temp_manifest(content: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("failed to create temp file");
    file.write_all(content.as_bytes())
        .expect("failed to write temp manifest");
    file.flush().expect("failed to flush temp manifest");
    file
}

fn pgpolicy_cmd() -> assert_cmd::Command {
    cargo_bin_cmd!("pgpolicy")
}

// ---------------------------------------------------------------------------
// Manifest fixtures
// ---------------------------------------------------------------------------

const VALID_MINIMAL: &str = r#"
default_owner: pgloader_pg

roles:
  - name: analytics
    login: true
    comment: "Analytics read-only role"

grants:
  - role: analytics
    privileges: [CONNECT]
    on: { type: database, name: mydb }
"#;

const VALID_PROFILES: &str = r#"
default_owner: pgloader_pg

profiles:
  editor:
    grants:
      - privileges: [USAGE]
        on: { type: schema }
      - privileges: [SELECT, INSERT, UPDATE, DELETE]
        on: { type: table, name: "*" }
    default_privileges:
      - privileges: [SELECT, INSERT, UPDATE, DELETE]
        on_type: table
  viewer:
    grants:
      - privileges: [USAGE]
        on: { type: schema }
      - privileges: [SELECT]
        on: { type: table, name: "*" }
    default_privileges:
      - privileges: [SELECT]
        on_type: table

schemas:
  - name: ibody
    profiles: [editor, viewer]
  - name: catalog
    profiles: [viewer]

roles:
  - name: app-service
    login: true

grants:
  - role: app-service
    privileges: [CONNECT]
    on: { type: database, name: mydb }

memberships:
  - role: ibody-editor
    members:
      - name: app-service
"#;

const INVALID_YAML: &str = r#"
this is: [not: valid yaml: [[
"#;

const UNDEFINED_PROFILE: &str = r#"
profiles:
  editor:
    grants: []

schemas:
  - name: myschema
    profiles: [nonexistent]
"#;

const EMPTY_MANIFEST: &str = r#"
roles: []
"#;

// =========================================================================
// validate subcommand
// =========================================================================

#[test]
fn validate_valid_minimal_manifest() {
    let manifest_file = write_temp_manifest(VALID_MINIMAL);

    pgpolicy_cmd()
        .args(["validate", "--file", manifest_file.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Manifest is valid"))
        .stdout(predicate::str::contains("1 role(s) defined"))
        .stdout(predicate::str::contains("1 grant(s) defined"));
}

#[test]
fn validate_valid_profiles_manifest() {
    let manifest_file = write_temp_manifest(VALID_PROFILES);

    pgpolicy_cmd()
        .args(["validate", "--file", manifest_file.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Manifest is valid"))
        .stdout(predicate::str::contains("4 role(s) defined"));
}

#[test]
fn validate_empty_manifest() {
    let manifest_file = write_temp_manifest(EMPTY_MANIFEST);

    pgpolicy_cmd()
        .args(["validate", "--file", manifest_file.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Manifest is valid"))
        .stdout(predicate::str::contains("0 role(s) defined"));
}

#[test]
fn validate_invalid_yaml() {
    let manifest_file = write_temp_manifest(INVALID_YAML);

    pgpolicy_cmd()
        .args(["validate", "--file", manifest_file.path().to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("YAML parse error"));
}

#[test]
fn validate_undefined_profile() {
    let manifest_file = write_temp_manifest(UNDEFINED_PROFILE);

    pgpolicy_cmd()
        .args(["validate", "--file", manifest_file.path().to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("nonexistent"));
}

#[test]
fn validate_nonexistent_file() {
    pgpolicy_cmd()
        .args([
            "validate",
            "--file",
            "/tmp/nonexistent-pgpolicy-test-xyz.yaml",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("failed to read manifest file"));
}

#[test]
fn validate_default_file_not_found() {
    // Running `pgpolicy validate` without --file should look for pgpolicy.yaml
    // in the current directory, which won't exist in a temp dir.
    pgpolicy_cmd()
        .current_dir(std::env::temp_dir())
        .args(["validate"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("failed to read manifest file"));
}

// =========================================================================
// Global CLI behaviour
// =========================================================================

#[test]
fn no_subcommand_shows_help() {
    // clap should show an error/help message when no subcommand is given.
    pgpolicy_cmd()
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage"));
}

#[test]
fn help_flag() {
    pgpolicy_cmd()
        .args(["--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("pgpolicy"))
        .stdout(predicate::str::contains("validate"))
        .stdout(predicate::str::contains("diff"))
        .stdout(predicate::str::contains("apply"))
        .stdout(predicate::str::contains("inspect"));
}

#[test]
fn version_flag() {
    pgpolicy_cmd()
        .args(["--version"])
        .assert()
        .success()
        .stdout(predicate::str::contains("pgpolicy"));
}

#[test]
fn validate_help() {
    pgpolicy_cmd()
        .args(["validate", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Validate"))
        .stdout(predicate::str::contains("--file"));
}

#[test]
fn diff_help() {
    pgpolicy_cmd()
        .args(["diff", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--database-url"))
        .stdout(predicate::str::contains("--file"))
        .stdout(predicate::str::contains("--format"));
}

#[test]
fn apply_help() {
    pgpolicy_cmd()
        .args(["apply", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--database-url"))
        .stdout(predicate::str::contains("--dry-run"));
}

#[test]
fn inspect_help() {
    pgpolicy_cmd()
        .args(["inspect", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--database-url"));
}

#[test]
fn plan_alias_for_diff() {
    // `plan` should be an alias for `diff` — verify it shows the same help.
    pgpolicy_cmd()
        .args(["plan", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--database-url"))
        .stdout(predicate::str::contains("--format"));
}

// =========================================================================
// diff/plan subcommand — requires DB (ignored by default)
// =========================================================================

#[test]
fn diff_missing_database_url() {
    let manifest_file = write_temp_manifest(VALID_MINIMAL);

    // No DATABASE_URL env var and no --database-url flag → should fail
    pgpolicy_cmd()
        .env_remove("DATABASE_URL")
        .args(["diff", "--file", manifest_file.path().to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("database-url"));
}

#[test]
fn apply_missing_database_url() {
    let manifest_file = write_temp_manifest(VALID_MINIMAL);

    pgpolicy_cmd()
        .env_remove("DATABASE_URL")
        .args(["apply", "--file", manifest_file.path().to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("database-url"));
}

#[test]
fn inspect_missing_database_url() {
    let manifest_file = write_temp_manifest(VALID_MINIMAL);

    pgpolicy_cmd()
        .env_remove("DATABASE_URL")
        .args(["inspect", "--file", manifest_file.path().to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("database-url"));
}

// =========================================================================
// diff/apply/inspect with invalid manifest (should fail before DB connect)
// =========================================================================

#[test]
fn diff_with_invalid_manifest() {
    let manifest_file = write_temp_manifest(INVALID_YAML);

    pgpolicy_cmd()
        .args([
            "diff",
            "--file",
            manifest_file.path().to_str().unwrap(),
            "--database-url",
            "postgres://localhost/test",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("YAML parse error"));
}

#[test]
fn apply_with_invalid_manifest() {
    let manifest_file = write_temp_manifest(INVALID_YAML);

    pgpolicy_cmd()
        .args([
            "apply",
            "--file",
            manifest_file.path().to_str().unwrap(),
            "--database-url",
            "postgres://localhost/test",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("YAML parse error"));
}

// =========================================================================
// Integration tests requiring a live database — #[ignore]d
// =========================================================================

/// These tests require a running PostgreSQL instance.
/// Set DATABASE_URL before running:
///   DATABASE_URL=postgres://localhost/pgpolicy_test cargo test -- --ignored
mod live_db {
    use super::*;

    fn database_url() -> String {
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for live DB tests")
    }

    #[test]
    #[ignore]
    fn diff_against_live_db() {
        let manifest_file = write_temp_manifest(VALID_MINIMAL);

        pgpolicy_cmd()
            .args([
                "diff",
                "--file",
                manifest_file.path().to_str().unwrap(),
                "--database-url",
                &database_url(),
            ])
            .assert()
            .success();
    }

    #[test]
    #[ignore]
    fn diff_summary_format() {
        let manifest_file = write_temp_manifest(VALID_MINIMAL);

        pgpolicy_cmd()
            .args([
                "diff",
                "--file",
                manifest_file.path().to_str().unwrap(),
                "--database-url",
                &database_url(),
                "--format",
                "summary",
            ])
            .assert()
            .success();
    }

    #[test]
    #[ignore]
    fn apply_dry_run_against_live_db() {
        let manifest_file = write_temp_manifest(VALID_MINIMAL);

        pgpolicy_cmd()
            .args([
                "apply",
                "--file",
                manifest_file.path().to_str().unwrap(),
                "--database-url",
                &database_url(),
                "--dry-run",
            ])
            .assert()
            .success();
    }

    #[test]
    #[ignore]
    fn inspect_against_live_db() {
        let manifest_file = write_temp_manifest(VALID_MINIMAL);

        pgpolicy_cmd()
            .args([
                "inspect",
                "--file",
                manifest_file.path().to_str().unwrap(),
                "--database-url",
                &database_url(),
            ])
            .assert()
            .success()
            .stdout(predicate::str::contains("Roles:"))
            .stdout(predicate::str::contains("Grants:"));
    }
}
