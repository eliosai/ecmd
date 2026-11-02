//! Scanning and help rendering for one GNU command

#![expect(dead_code, reason = "the fixture parses fields it never reads")]

use divan::{Bencher, black_box};
use ecmd::{Command, Operands};

fn main() {
    divan::main();
}

/// List directory contents
#[derive(Command)]
#[command(name = "ls", style = "gnu")]
struct List {
    /// do not ignore entries starting with .
    #[flag(short = 'a')]
    all: bool,
    /// use a long listing format
    #[flag(short = 'l')]
    long: bool,
    /// print sizes like 1K 234M 2G
    #[flag(short = 'h', long = "human-readable")]
    human: bool,
    /// sort by WORD instead of name
    #[flag(long = "sort", value_name = "WORD")]
    sort: Option<String>,
    /// colorize the output
    #[flag(long = "color", value_name = "WHEN")]
    color: Option<String>,
    files: Operands,
}

#[divan::bench]
fn parse_short_cluster_and_longs(bencher: Bencher) {
    let args = ["-alh", "--sort=time", "--color", "never", "src", "docs"];
    bencher.bench(|| List::parse(black_box(&args)).is_ok());
}

#[divan::bench]
fn read_definition(bencher: Bencher) {
    bencher.bench(|| black_box(List::def()).flags().len());
}

#[divan::bench]
fn render_help(bencher: Bencher) {
    bencher.bench(|| black_box(List::def()).help());
}
