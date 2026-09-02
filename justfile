# ecmd justfile
# Usage: just <command> [args]

# List every command
default:
    @just --list

# Scan comments, docs and layout, then format-check, type-check and lint every feature set
check:
    bash scripts/comment-scan.sh
    bash scripts/doc-scan.sh
    bash scripts/layout-scan.sh
    cargo fmt --all -- --check
    cargo check --workspace --all-targets --all-features
    cargo check -p ecmd --no-default-features
    cargo check -p ecmd --no-default-features --features rkyv
    cargo clippy --workspace --all-targets --all-features -- -D warnings

# Format the workspace
fmt:
    cargo fmt --all

# Build every target with every feature
build:
    cargo build --workspace --all-targets --all-features

# Run the test suite
test:
    cargo nextest run --workspace --all-features

# Run the test suite the way the gate does
test-ci:
    cargo nextest run --profile ci --workspace --all-features

# Run every doc example, which nextest cannot
test-doc:
    cargo test --workspace --all-features --doc

# Build the docs the way docs.rs does, failing on any warning or broken link
doc-check:
    RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps

# Build the docs and open them
docs-open:
    RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps --open

# Type-check on the minimum supported Rust version the manifests declare
msrv:
    cargo +1.88 check --workspace --all-features --locked

# Compare the public API against the last published release
semver-check baseline="":
    cargo semver-checks --workspace --all-features {{ if baseline != "" { "--baseline-rev " + baseline } else { "" } }}

# Build the crates.io packages and list what ships in each
package-check:
    cargo package --workspace --locked --all-features --allow-dirty
    cargo package --workspace --locked --all-features --allow-dirty --list

# Check licenses, advisories, duplicate versions and sources
audit:
    cargo deny check

# Install the git hooks
hooks:
    prek install --hook-type pre-commit --hook-type pre-push

# Run the hooks against every file
hooks-run:
    prek run --all-files

# Print the version the next merge to main would release
release-plan:
    bash scripts/release.sh --dry-run

# Run everything the gate runs
ci:
    just check
    just test-ci
    just test-doc
    just doc-check
    just package-check
    just audit

# Remove every build artifact
clean:
    cargo clean
