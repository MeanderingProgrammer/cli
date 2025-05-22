mod common;

use std::collections::HashSet;
use std::path::PathBuf;

use common::TestData;
use copy_folder::reader::Reader;

#[test]
fn test_empty() {
    let files = [];
    test(&files, &files);
}

#[test]
fn test_file() {
    let files = ["file.txt"];
    test(&files, &files);
}

#[test]
fn test_nested_file() {
    let files = ["a/file.txt"];
    test(&files, &files);
}

#[test]
fn test_many() {
    let files = [
        "file-1.txt",
        "file-2.txt",
        "a/file-1.txt",
        "a/file-2.txt",
        "a/b/files-1.txt",
        "a/b/c/files-1.txt",
        "a/b/c/d/e/f/g/h/files-1.txt",
    ];
    test(&files, &files);
}

#[test]
fn test_ignore() {
    test(
        &[
            "file.txt",
            "._file.txt",
            ".DS_Store",
            "a/file.txt",
            "a/._file.txt",
            "a/.DS_Store",
        ],
        &["file.txt", "a/file.txt"],
    );
}

fn test(files: &[&str], expected: &[&str]) {
    let data = TestData::new(files);
    let expected: HashSet<_> = expected.iter().map(PathBuf::from).collect();
    let reader = Reader::new([".DS_Store".into()], ["._".into()]);
    assert_eq!(expected, reader.get(data.path()).unwrap());
}
