# ecmd justfile
# Usage: just <command> [args]

# List every command
default:
    @just --list

# Run every file level check, which is the same set prek runs on a commit and ci runs on a push
lint:
    prek run --all-files

# Format check, then compile every feature pair, then lint every target
# Clippy runs the same front end as `cargo check`, so no plain check pass runs beside it
check:
    cargo fmt --all -- --check
    cargo hack check -p ecmd --feature-powerset --depth 2 --no-dev-deps
    cargo clippy --workspace --all-targets --all-features -- -D warnings

# Compile every feature subset of the library crate, which the paired sweep in `check` bounds at two
features:
    cargo hack check -p ecmd --feature-powerset --no-dev-deps

# Ask whether the lower bounds the manifests declare actually resolve and build
minimal:
    cargo minimal-versions check --workspace --all-features --direct

# Name every dependency no crate in the workspace reaches
unused:
    cargo machete --with-metadata

# Report line coverage over the same run the gate makes, which is a figure to read and never a gate
coverage:
    cargo llvm-cov nextest --profile ci --workspace --all-features --lcov --output-path lcov.info

# Name every mutant that no test noticed, bounded to what this branch changed
mutants base="origin/main":
    git diff {{base}}... > /tmp/ecmd-mutants.diff
    cargo mutants --test-tool=nextest --workspace --in-diff /tmp/ecmd-mutants.diff

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

# Compare the public API against the last release, or against the given revision
semver-check baseline="":
    bash scripts/semver-check.sh {{baseline}}

# Build the crates.io packages and list what ships in each
package-check:
    cargo package --workspace --locked --all-features --allow-dirty
    cargo package --workspace --locked --all-features --allow-dirty --list

# Check licenses, advisories, duplicate versions and sources
audit:
    cargo deny check

# Install the git hooks
hooks:
    prek install --prepare-hooks

# Run the hooks against every file
hooks-run:
    prek run --all-files

# Print the version the next merge to main would release
release-plan:
    bash scripts/release.sh --dry-run

# Run everything the pull request gate runs
ci:
    just lint
    just check
    just test-ci
    just test-doc
    just doc-check
    just package-check
    just audit
    just unused
    just msrv

# Remove every build artifact
clean:
    cargo clean
