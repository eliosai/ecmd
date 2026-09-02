//! Doc comments split into the help sections bash prints

use syn::{Expr, Lit};

/// Parsed doc comment split into help sections.
pub struct DocSections {
    pub about: String,
    pub description: Vec<String>,
    pub extra: Vec<String>,
    pub exit_status: Vec<String>,
}

/// Extract and split doc comment into bash-compatible help sections.
pub fn extract_doc_sections(attrs: &[syn::Attribute]) -> DocSections {
    let lines = extract_doc_lines(attrs);
    parse_sections(&lines)
}

/// Extract doc comment lines from attributes, trimmed and joined.
pub fn extract_doc_comment(attrs: &[syn::Attribute]) -> String {
    let lines = extract_doc_lines(attrs);
    lines.join("\n").trim().to_owned()
}

fn extract_doc_lines(attrs: &[syn::Attribute]) -> Vec<String> {
    attrs
        .iter()
        .filter(|a| a.path().is_ident("doc"))
        .filter_map(|a| {
            if let syn::Meta::NameValue(nv) = &a.meta
                && let Expr::Lit(syn::ExprLit {
                    lit: Lit::Str(s), ..
                }) = &nv.value
            {
                return Some(s.value());
            }
            None
        })
        .map(|s| s.strip_prefix(' ').unwrap_or(&s).to_owned())
        // an exact bare fence keeps rustdoc from reading an indented help block as a doctest
        .filter(|line| line != "```text" && line != "```")
        .collect()
}

#[derive(PartialEq)]
enum DocState {
    About,
    Description,
    Extra,
    ExitStatus,
}

fn parse_sections(lines: &[String]) -> DocSections {
    let mut about = String::new();
    let mut description = Vec::new();
    let mut extra = Vec::new();
    let mut exit_status = Vec::new();

    let mut state = DocState::About;
    let mut past_about_blank = false;

    for line in lines {
        let trimmed = line.trim();

        if trimmed == "# Options" {
            state = DocState::Extra;
            continue;
        }
        if trimmed == "# Exit Status" {
            state = DocState::ExitStatus;
            continue;
        }

        match state {
            DocState::About => {
                if trimmed.is_empty() {
                    if !about.is_empty() {
                        past_about_blank = true;
                    }
                } else if past_about_blank {
                    state = DocState::Description;
                    description.push(trimmed.to_owned());
                } else if about.is_empty() {
                    trimmed.clone_into(&mut about);
                } else {
                    about.push('\n');
                    about.push_str(trimmed);
                }
            }
            DocState::Description => {
                description.push(if trimmed.is_empty() {
                    String::new()
                } else {
                    line.clone()
                });
            }
            DocState::Extra => {
                extra.push(if trimmed.is_empty() {
                    String::new()
                } else {
                    line.clone()
                });
            }
            DocState::ExitStatus => {
                exit_status.push(if trimmed.is_empty() {
                    String::new()
                } else {
                    line.clone()
                });
            }
        }
    }

    trim_trailing_empty(&mut description);
    trim_trailing_empty(&mut extra);
    trim_trailing_empty(&mut exit_status);

    DocSections {
        about,
        description,
        extra,
        exit_status,
    }
}

fn trim_trailing_empty(lines: &mut Vec<String>) {
    while lines.last().is_some_and(String::is_empty) {
        lines.pop();
    }
}
