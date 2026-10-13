//! Unit tests for repository path validation and ordering.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use super::RepoPath;

fn rule(path: &str) -> &'static str {
    RepoPath::new(path).unwrap_err().rule
}

#[test]
fn accepts_nested_utf8_paths() {
    let path = RepoPath::new("docs/größe/readme.md").unwrap();
    assert_eq!(path.as_str(), "docs/größe/readme.md");
    assert_eq!(path.to_string(), "docs/größe/readme.md");
    assert_eq!(path.components().count(), 3);
}

#[test]
fn rejects_the_root_and_empty_components() {
    assert_eq!(rule(""), "empty component");
    assert_eq!(rule("/a"), "empty component");
    assert_eq!(rule("a/"), "empty component");
    assert_eq!(rule("a//b"), "empty component");
}

#[test]
fn rejects_dot_components() {
    assert_eq!(rule("./a"), "`.` or `..` component");
    assert_eq!(rule("a/../b"), "`.` or `..` component");
}

#[test]
fn rejects_nul() {
    assert_eq!(rule("a\0b"), "component contains NUL");
}

#[test]
fn rejects_overlong_components_and_paths() {
    assert_eq!(rule(&"a".repeat(256)), "component longer than 255 bytes");
    assert!(RepoPath::new("a".repeat(255)).is_ok());
    let long = vec!["a".repeat(200); 21].join("/");
    assert_eq!(rule(&long), "path longer than 4096 bytes");
}

#[test]
fn a_single_component_cannot_contain_a_separator() {
    assert_eq!(
        super::types::component_violation("a/b"),
        Some("component contains `/`")
    );
}

#[test]
fn orders_by_utf8_bytes() {
    let mut paths = ["b", "a/z", "a.txt", "B"].map(|p| RepoPath::new(p).unwrap());
    paths.sort();
    let sorted: Vec<_> = paths.iter().map(RepoPath::as_str).collect();
    assert_eq!(sorted, ["B", "a.txt", "a/z", "b"]);
}

#[test]
fn errors_name_the_path_and_rule() {
    let error = RepoPath::new("a//b").unwrap_err();
    assert_eq!(error.to_string(), "invalid path \"a//b\": empty component");
}

#[test]
fn serializes_as_a_string_and_validates_on_decode() {
    let path = RepoPath::new("src/main.rs").unwrap();
    assert_eq!(
        serde_json::to_value(&path).unwrap(),
        serde_json::json!("src/main.rs")
    );
    let decoded: RepoPath = serde_json::from_value(serde_json::json!("src/main.rs")).unwrap();
    assert_eq!(decoded, path);
    assert!(serde_json::from_value::<RepoPath>(serde_json::json!("../x")).is_err());
}
