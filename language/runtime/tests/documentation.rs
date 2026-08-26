// SPDX-License-Identifier: AGPL-3.0-or-later
// Rust guideline compliant 2026-02-21

use std::fs;
use std::path::{Path, PathBuf};

use native_space_language::parse_document;

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn markdown_files(directory: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            if path.ends_with("target") || path.ends_with("private") {
                continue;
            }
            markdown_files(&path, files);
        } else if path.extension().and_then(|value| value.to_str()) == Some("md") {
            files.push(path);
        }
    }
}

#[test]
fn every_native_space_markdown_block_is_a_complete_document() {
    let root = repository();
    let mut files = Vec::new();
    markdown_files(&root, &mut files);

    for path in files {
        let document = fs::read_to_string(&path).unwrap();
        let mut block = None::<(usize, String)>;
        for (index, line) in document.lines().enumerate() {
            if line.trim() == "```ns" {
                block = Some((index + 2, String::new()));
                continue;
            }
            let Some((start_line, source)) = &mut block else {
                continue;
            };
            if line.trim() == "```" {
                let source_name = format!("{}:{start_line}", path.display());
                parse_document(source, &source_name)
                    .unwrap_or_else(|error| panic!("{source_name}: {error}"));
                block = None;
            } else {
                source.push_str(line);
                source.push('\n');
            }
        }
        assert!(
            block.is_none(),
            "unclosed Native Space block: {}",
            path.display()
        );
    }
}

#[test]
fn every_markdown_file_uses_supported_github_math_syntax() {
    let root = repository();
    let mut files = Vec::new();
    markdown_files(&root, &mut files);

    for path in files {
        let document = fs::read_to_string(&path).unwrap();
        for unsupported in [r"\(", r"\)", r"\[", r"\]"] {
            assert!(
                !document.contains(unsupported),
                "unsupported math delimiter {unsupported:?}: {}",
                path.display()
            );
        }
        for ambiguous in [
            r"\operatorname{",
            r"\mathbb ",
            r"\frac12",
            r"\tfrac12",
            r"\sqrt2",
            r"\zeta\!",
        ] {
            assert!(
                !document.contains(ambiguous),
                "unsupported or ambiguous GitHub math {ambiguous:?}: {}",
                path.display()
            );
        }
        assert_eq!(
            document.matches('$').count() % 2,
            0,
            "unbalanced dollar math delimiters: {}",
            path.display()
        );
    }
}
